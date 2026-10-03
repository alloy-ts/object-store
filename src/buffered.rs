use crate::store::ObjectStore as NapiObjectStore;
use crate::types::{build_object_meta, ObjectMeta};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::buffered::{BufReader as RSBufReader, BufWriter as RSBufWriter};
use object_store::path::Path;
use std::io::{Error as IoError, SeekFrom};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

fn io_err(e: IoError) -> napi::Error {
  napi::Error::from_reason(e.to_string())
}

fn rs_err(e: object_store::Error) -> napi::Error {
  napi::Error::from_reason(e.to_string())
}

/// Mirrors [`std::io::SeekFrom`] for the JS side: where to seek from and by how
/// much. `offset` is signed because `"end"` / `"current"` may be negative, and is
/// a `bigint` because object offsets are 64-bit.
#[napi(object)]
pub struct SeekFromInput {
  /// One of `"start"`, `"end"`, `"current"`.
  pub kind: String,
  /// Offset in bytes; may be negative for `"end"` / `"current"`.
  pub offset: i64,
}

fn build_seek(seek: SeekFromInput) -> napi::Result<SeekFrom> {
  match seek.kind.as_str() {
    "start" => {
      if seek.offset < 0 {
        return Err(napi::Error::from_reason(
          "SeekFrom::Start requires a non-negative offset".to_string(),
        ));
      }
      Ok(SeekFrom::Start(seek.offset as u64))
    }
    "end" => Ok(SeekFrom::End(seek.offset)),
    "current" => Ok(SeekFrom::Current(seek.offset)),
    other => Err(napi::Error::from_reason(format!(
      "invalid seek kind '{other}', expected start|end|current"
    ))),
  }
}

/// NAPI binding for [`object_store::buffered::BufReader`].
///
/// An async-buffered reader that batches reads through `get_range`, clearing its
/// internal buffer on seek. The underlying reader is kept behind a tokio `Mutex`
/// so the `&self` async methods can drive the `poll_*`-based tokio IO traits.
#[napi]
pub struct BufReader {
  inner: Arc<tokio::sync::Mutex<RSBufReader>>,
  size: u64,
}

#[napi]
impl BufReader {
  /// Create a `BufReader` using the default 1 MiB internal buffer.
  #[napi(factory)]
  pub fn new(store: &NapiObjectStore, meta: ObjectMeta) -> Self {
    Self::with_capacity(store, meta, object_store::buffered::DEFAULT_BUFFER_SIZE as u32)
  }

  /// Create a `BufReader` with an explicit internal buffer `capacity` (bytes).
  #[napi(factory)]
  pub fn with_capacity(store: &NapiObjectStore, meta: ObjectMeta, capacity: u32) -> Self {
    let rs_meta = build_object_meta(&meta);
    let reader = RSBufReader::with_capacity(store.inner.clone(), &rs_meta, capacity as usize);
    Self {
      size: rs_meta.size,
      inner: Arc::new(tokio::sync::Mutex::new(reader)),
    }
  }

  /// Create a `BufReader` from a store, metadata, and optional buffered options.
  #[napi(factory)]
  pub fn create(store: &NapiObjectStore, meta: ObjectMeta, options: Option<BufWriterOptions>) -> Self {
    let capacity = options
      .and_then(|o| o.capacity)
      .unwrap_or(object_store::buffered::DEFAULT_BUFFER_SIZE as u32);
    Self::with_capacity(store, meta, capacity)
  }

  /// Total size of the underlying object in bytes.
  #[napi]
  pub fn size(&self) -> f64 {
    self.size as f64
  }

  /// Read up to `size` bytes, returning the bytes actually read (empty at EOF).
  #[napi]
  pub async fn read(&self, size: u32) -> napi::Result<Buffer> {
    let mut guard = self.inner.lock().await;
    let mut buf = vec![0u8; size as usize];
    let n = AsyncReadExt::read(&mut *guard, &mut buf).await.map_err(io_err)?;
    buf.truncate(n);
    Ok(Buffer::from(buf))
  }

  /// Return the current contents of the internal buffer without consuming it.
  #[napi]
  pub async fn fill_buf(&self) -> napi::Result<Buffer> {
    let mut guard = self.inner.lock().await;
    let buf = AsyncBufReadExt::fill_buf(&mut *guard).await.map_err(io_err)?;
    Ok(Buffer::from(buf.to_vec()))
  }

  /// Mark `amt` bytes of the internal buffer as consumed, advancing the cursor.
  #[napi]
  pub async fn consume(&self, amt: u32) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    AsyncBufReadExt::consume(&mut *guard, amt as usize);
    Ok(())
  }

  /// Seek to a position and return the absolute stream position afterwards.
  #[napi]
  pub async fn seek(&self, seek: SeekFromInput) -> napi::Result<i64> {
    let mut guard = self.inner.lock().await;
    let pos = AsyncSeekExt::seek(&mut *guard, build_seek(seek)?)
      .await
      .map_err(io_err)?;
    Ok(pos as i64)
  }

  /// Return the current absolute stream position.
  #[napi]
  pub async fn stream_position(&self) -> napi::Result<i64> {
    let mut guard = self.inner.lock().await;
    let pos = AsyncSeekExt::stream_position(&mut *guard)
      .await
      .map_err(io_err)?;
    Ok(pos as i64)
  }
}

/// Options for constructing a [`BufWriter`].
#[napi(object)]
pub struct BufWriterOptions {
  /// Internal buffer capacity in bytes (defaults to 10 MiB).
  pub capacity: Option<u32>,
  /// Maximum number of in-flight multipart requests (defaults to 8).
  pub max_concurrency: Option<u32>,
}

/// NAPI binding for [`object_store::buffered::BufWriter`].
///
/// Adaptively buffers in memory and flushes via `put_opts`, or streams via
/// `put_multipart_opts` once `capacity` is exceeded. Wrapped in a tokio `Mutex`
/// so the `&self` async methods can drive the tokio `AsyncWrite` trait.
#[napi]
pub struct BufWriter {
  inner: Arc<tokio::sync::Mutex<RSBufWriter>>,
}

#[napi]
impl BufWriter {
  /// Create a `BufWriter` with the default 10 MiB buffer.
  #[napi(factory)]
  pub fn new(store: &NapiObjectStore, path: String) -> Self {
    Self::create(store, path, None)
  }

  /// Create a `BufWriter` with an explicit `capacity` (defaults to 10 MiB).
  #[napi(factory)]
  pub fn with_capacity(store: &NapiObjectStore, path: String, capacity: u32) -> Self {
    Self::create(
      store,
      path,
      Some(BufWriterOptions {
        capacity: Some(capacity),
        max_concurrency: None,
      }),
    )
  }

  /// Create a `BufWriter` from a store, path, and optional buffered options.
  #[napi(factory)]
  pub fn create(store: &NapiObjectStore, path: String, options: Option<BufWriterOptions>) -> Self {
    let (capacity, max_concurrency) = match options {
      Some(o) => (
        o.capacity.unwrap_or(10 * 1024 * 1024) as usize,
        o.max_concurrency.unwrap_or(8) as usize,
      ),
      None => (10 * 1024 * 1024, 8),
    };
    let writer = RSBufWriter::with_capacity(
      store.inner.clone(),
      Path::from(path.as_str()),
      capacity,
    )
    .with_max_concurrency(max_concurrency);
    Self {
      inner: Arc::new(tokio::sync::Mutex::new(writer)),
    }
  }

  /// Write `data` to the buffer, returning the number of bytes accepted.
  #[napi]
  pub async fn write(&self, data: Buffer) -> napi::Result<u32> {
    let mut guard = self.inner.lock().await;
    let n = AsyncWriteExt::write(&mut *guard, data.as_ref())
      .await
      .map_err(io_err)?;
    Ok(n as u32)
  }

  /// Write `data` without extra copying. Recommended when the caller already
  /// holds the bytes.
  #[napi]
  pub async fn put(&self, data: Buffer) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    guard
      .put(bytes::Bytes::from(data.to_vec()))
      .await
      .map_err(rs_err)?;
    Ok(())
  }

  /// Flush buffered data. Does not finalize the upload.
  #[napi]
  pub async fn flush(&self) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    AsyncWriteExt::flush(&mut *guard).await.map_err(io_err)?;
    Ok(())
  }

  /// Finalize the upload (flush + shutdown). Required to persist the data.
  #[napi]
  pub async fn shutdown(&self) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    AsyncWriteExt::shutdown(&mut *guard).await.map_err(io_err)?;
    Ok(())
  }

  /// Abort the writer, cleaning up any partially uploaded multipart state.
  #[napi]
  pub async fn abort(&self) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    guard.abort().await.map_err(rs_err)?;
    Ok(())
  }
}
