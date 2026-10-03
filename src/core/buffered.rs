use std::io::SeekFrom;
use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::buffered::{BufReader as ObjBufReader, BufWriter as ObjBufWriter};
use object_store::path::Path;
use object_store::ObjectStoreExt;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

use super::put::PutResult;
use super::types::ObjectStore;

#[napi]
pub struct BufReader {
  inner: Arc<Mutex<ObjBufReader>>,
}

#[napi]
impl BufReader {
  #[napi(factory)]
  pub async fn open(store: &ObjectStore, path: String, capacity: Option<i64>) -> Result<BufReader> {
    let location = Path::from(path.as_str());
    let meta = store
      .inner
      .head(&location)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;

    let reader = if let Some(cap) = capacity {
      if cap > 0 {
        ObjBufReader::with_capacity(store.inner.clone(), &meta, cap as usize)
      } else {
        ObjBufReader::new(store.inner.clone(), &meta)
      }
    } else {
      ObjBufReader::new(store.inner.clone(), &meta)
    };

    Ok(BufReader {
      inner: Arc::new(Mutex::new(reader)),
    })
  }

  #[napi]
  pub async fn read(&self, size: i64) -> Result<Buffer> {
    let mut buf = vec![0u8; size.max(0) as usize];
    let mut guard = self.inner.lock().await;
    let n = guard
      .read(&mut buf)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    buf.truncate(n);
    Ok(Buffer::from(buf))
  }

  #[napi]
  pub async fn seek(&self, position: i64) -> Result<i64> {
    let mut guard = self.inner.lock().await;
    let new_pos = guard
      .seek(SeekFrom::Start(position.max(0) as u64))
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(new_pos as i64)
  }
}

#[napi]
pub struct BufWriter {
  inner: Arc<Mutex<Option<ObjBufWriter>>>,
}

#[napi]
impl BufWriter {
  #[napi(constructor)]
  pub fn new(store: &ObjectStore, path: String) -> Self {
    let location = Path::from(path.as_str());
    let writer = ObjBufWriter::new(store.inner.clone(), location);
    Self {
      inner: Arc::new(Mutex::new(Some(writer))),
    }
  }

  #[napi]
  pub async fn write(&self, #[napi(ts_arg_type = "Uint8Array | Buffer")] chunk: Uint8Array) -> Result<()> {
    let mut guard = self.inner.lock().await;
    if let Some(writer) = guard.as_mut() {
      writer
        .write_all(chunk.as_ref())
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(())
    } else {
      Err(Error::from_reason("BufWriter has already been finished".to_string()))
    }
  }

  #[napi]
  pub async fn finish(&self) -> Result<PutResult> {
    let mut guard = self.inner.lock().await;
    if let Some(mut writer) = guard.take() {
      writer
        .shutdown()
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(PutResult {
        e_tag: None,
        version: None,
      })
    } else {
      Err(Error::from_reason("BufWriter has already been finished".to_string()))
    }
  }
}
