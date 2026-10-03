use crate::store::ObjectStore;
use crate::types::{build_put_options, PutOptionsInput, PutResult};
use bytes::Bytes;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::MultipartUpload as MultipartUploadTrait;
use object_store::path::Path;
use object_store::{ObjectStoreExt, PutPayload};
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
    data: Buffer,
    options: Option<PutOptionsInput>,
  ) -> napi::Result<PutResult> {
    let location = Path::from(path.as_str());
    let payload = PutPayload::from(Bytes::from(data.to_vec()));

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
    data: Buffer,
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
  pub async fn put_part(&self, data: Buffer) -> napi::Result<()> {
    let payload = PutPayload::from(Bytes::from(data.to_vec()));
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
