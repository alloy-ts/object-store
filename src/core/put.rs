use bytes::Bytes;
use crate::store::ObjectStore;
use crate::types::{build_put_options, PutOptionsInput, PutResult};
use napi::bindgen_prelude::{Buffer, Either};
use napi_derive::napi;
use object_store::MultipartUpload as MultipartUploadTrait;
use object_store::path::Path;
use object_store::ObjectStoreExt;
use object_store::PutPayload as RsPutPayload;
use object_store::PutPayloadMut as RsPutPayloadMut;
use std::sync::Arc;
use tokio::sync::Mutex;

#[napi]
impl ObjectStore {
  /// Save the provided bytes to `path`.
  ///
  /// Wraps `ObjectStoreExt::put`. The payload is buffered in memory; use
  /// `putMultipart` for streaming uploads. Passing `options` routes through
  /// `ObjectStore::put_opts` (Overwrite/Create/Update modes).
  #[napi]
  pub async fn put(
    &self,
    path: String,
    data: Either<Buffer, &PutPayload>,
    options: Option<PutOptionsInput>,
  ) -> napi::Result<PutResult> {
    let location = Path::from(path.as_str());
    let payload = to_put_payload(data);

    if let Some(opts) = options {
      let put_options = build_put_options(&opts);
      let res = self
        .inner
        .put_opts(&location, payload, put_options)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      return Ok(PutResult {
        e_tag: res.e_tag,
        version: res.version,
      });
    }

    let res = self
      .inner
      .put(&location, payload)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  /// Save the provided bytes to `path` with the given options.
  ///
  /// Wraps `ObjectStore::put_opts` (Overwrite/Create/Update modes). The
  /// operation is atomic. For no-option writes see `put`; for streaming
  /// uploads see `putMultipart`.
  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    data: Either<Buffer, &PutPayload>,
    options: PutOptionsInput,
  ) -> napi::Result<PutResult> {
    self.put(path, data, Some(options)).await
  }

  /// Start a multipart upload, returning a handle to feed parts into.
  ///
  /// Wraps `ObjectStoreExt::put_multipart`. Prefer `put` for small payloads.
  #[napi]
  pub async fn put_multipart(&self, path: String) -> napi::Result<MultipartUpload> {
    let location = Path::from(path.as_str());
    let upload = self
      .inner
      .put_multipart(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(MultipartUpload {
      inner: Arc::new(Mutex::new(upload)),
    })
  }
}

/// Handle to an in-progress multipart upload, returned by
/// `ObjectStore.putMultipart`.
///
/// Upload parts in order with `putPart`, then finish with `complete` or
/// discard with `abort`.
#[napi]
pub struct MultipartUpload {
  pub(crate) inner: Arc<Mutex<Box<dyn MultipartUploadTrait>>>,
}

#[napi]
impl MultipartUpload {
  /// Upload the next part. Parts are identified by call order; call
  /// `complete` once all parts have been uploaded.
  #[napi]
  pub async fn put_part(&self, data: Either<Buffer, &PutPayload>) -> napi::Result<()> {
    let payload = to_put_payload(data);
    let mut guard = self.inner.lock().await;
    guard
      .put_part(payload)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  /// Complete the multipart upload.
  #[napi]
  pub async fn complete(&self) -> napi::Result<PutResult> {
    let mut guard = self.inner.lock().await;
    let res = guard
      .complete()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  /// Abort the multipart upload, discarding any uploaded parts.
  #[napi]
  pub async fn abort(&self) -> napi::Result<()> {
    let mut guard = self.inner.lock().await;
    guard
      .abort()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}

#[derive(Clone)]
#[napi]
pub struct PutPayload {
  pub(crate) inner: RsPutPayload,
}

#[napi]
impl PutPayload {
  /// Create a new empty `PutPayload`.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsPutPayload::new(),
    }
  }

  /// Build a `PutPayload` from a single buffer (copied into a `Bytes`).
  #[napi(factory)]
  pub fn from_bytes(data: Buffer) -> Self {
    Self {
      inner: RsPutPayload::from(Bytes::from(data.to_vec())),
    }
  }

  /// Build a `PutPayload` from a UTF-8 string.
  #[napi(factory)]
  pub fn from_string(data: String) -> Self {
    Self {
      inner: RsPutPayload::from(data),
    }
  }

  /// Total number of bytes across all chunks.
  #[napi]
  pub fn content_length(&self) -> f64 {
    self.inner.content_length() as f64
  }

  /// Whether this payload holds no bytes.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.content_length() == 0
  }

  /// Return each underlying chunk as a separate `Buffer`.
  ///
  /// A `PutPayload` is an ordered collection of `Bytes`; this exposes the
  /// individual chunks without reallocating.
  #[napi]
  pub fn chunks(&self) -> Vec<Buffer> {
    self
      .inner
      .as_ref()
      .iter()
      .map(|b| Buffer::from(b.as_ref()))
      .collect()
  }

  /// Concatenate every chunk into a single `Buffer`.
  #[napi]
  pub fn concat(&self) -> Buffer {
    Buffer::from(Bytes::from(self.inner.clone()).as_ref())
  }

  /// Cheaply clone the payload (shares the underlying `Bytes` via `Arc`).
  #[napi]
  pub fn clone(&self) -> PutPayload {
    PutPayload {
      inner: self.inner.clone(),
    }
  }
}

/// NAPI binding for `object_store::PutPayloadMut`.
///
/// A builder for [`PutPayload`] that avoids reallocating memory: data is
/// accumulated in fixed blocks (default 8 KiB), flushed to `Bytes` once full.
/// Call `freeze` to obtain an immutable [`PutPayload`] (which is what `put` /
/// `putPart` accept).
#[napi]
pub struct PutPayloadMut {
  pub(crate) inner: RsPutPayloadMut,
}

#[napi]
impl PutPayloadMut {
  /// Create a new `PutPayloadMut` with the default 8 KiB block size.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsPutPayloadMut::new(),
    }
  }

  /// Create a `PutPayloadMut` with a custom minimum allocation block size.
  ///
  /// Must be called before any data is written.
  #[napi(factory)]
  pub fn with_block_size(block_size: u32) -> Self {
    Self {
      inner: RsPutPayloadMut::new().with_block_size(block_size as usize),
    }
  }

  /// Append `data` as a new `Bytes` chunk without copying the underlying data
  /// again. Closes any in-progress block first.
  #[napi]
  pub fn push(&mut self, data: Buffer) {
    self.inner.push(Bytes::from(data.to_vec()));
  }

  /// Write `data` into this payload using the block-buffered allocator.
  #[napi]
  pub fn extend_from_slice(&mut self, data: Buffer) {
    self.inner.extend_from_slice(&data.to_vec());
  }

  /// Total number of bytes written so far.
  #[napi]
  pub fn content_length(&self) -> f64 {
    self.inner.content_length() as f64
  }

  /// Whether no bytes have been written.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.content_length() == 0
  }

  /// Freeze into an immutable [`PutPayload`]. This instance becomes empty.
  #[napi]
  pub fn freeze(&mut self) -> PutPayload {
    let mut tmp = RsPutPayloadMut::new();
    std::mem::swap(&mut self.inner, &mut tmp);
    PutPayload {
      inner: tmp.freeze(),
    }
  }
}

/// Accept either a raw `Buffer` or a `PutPayload` as a `put` payload.
///
/// Shared by every `put` / `putPart` binding so the conversion lives in one
/// place. (A `PutPayloadMut` must be `freeze`d to a `PutPayload` first.)
pub(crate) fn to_put_payload(data: Either<Buffer, &PutPayload>) -> RsPutPayload {
  match data {
    Either::A(buf) => RsPutPayload::from(Bytes::from(buf.to_vec())),
    Either::B(p) => p.inner.clone(),
  }
}
