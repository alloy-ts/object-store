use crate::memory::InMemory;
use crate::core::{to_put_payload, PutPayload};
use crate::types::{PutMultipartOptionsInput, PutResult};
use napi::bindgen_prelude::{Buffer, Either};
use napi_derive::napi;
use object_store::multipart::{MultipartStore as MultipartStoreTrait, PartId as RsPartId};
use object_store::path::Path;
use object_store::{MultipartId, PutMultipartOptions};
use std::sync::Arc;

/// A part of a file that has been successfully uploaded in a multipart upload process.
#[napi(object)]
pub struct PartId {
  pub content_id: String,
}

/// NAPI binding for the low-level [`object_store::multipart::MultipartStore`] trait.
///
/// This is the raw multipart API (create → put_part × N → complete / abort). Most
/// callers should prefer the higher-level `put_multipart_opts` flow, but this binding
/// is available for backends that implement `MultipartStore` (in this build: `InMemory`).
///
/// It wraps `Arc<dyn MultipartStore>`, so it can be backed by any backend that
/// implements the trait.
#[napi]
pub struct MultipartStore {
  pub(crate) inner: Arc<dyn MultipartStoreTrait>,
}

#[napi]
impl MultipartStore {
  /// Create a `MultipartStore` from an in-memory store.
  #[napi(factory)]
  pub fn from_in_memory(store: &InMemory) -> Self {
    // `InMemory` implements `MultipartStore`, so coerce `Arc<InMemory>` to
    // `Arc<dyn MultipartStore>`.
    let inner: Arc<dyn MultipartStoreTrait> = store.inner.clone();
    Self { inner }
  }

  /// Creates a new multipart upload, returning the `MultipartId`.
  #[napi]
  pub async fn create_multipart(&self, path: String) -> napi::Result<String> {
    let location = Path::from(path.as_str());
    let id: MultipartId = self
      .inner
      .create_multipart(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(id)
  }

  /// Creates a new multipart upload with the given options, returning the `MultipartId`.
  #[napi]
  pub async fn create_multipart_opts(
    &self,
    path: String,
    _options: Option<PutMultipartOptionsInput>,
  ) -> napi::Result<String> {
    let location = Path::from(path.as_str());
    // Only the default options are forwarded; `InMemory`'s default impl rejects
    // non-default attributes/tags, so this keeps parity with the underlying store.
    let opts = PutMultipartOptions::default();
    let id: MultipartId = self
      .inner
      .create_multipart_opts(&location, opts)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(id)
  }

  /// Uploads a new part with index `part_idx`.
  #[napi]
  pub async fn put_part(
    &self,
    path: String,
    id: String,
    part_idx: u32,
    data: Either<Buffer, &PutPayload>,
  ) -> napi::Result<PartId> {
    let location = Path::from(path.as_str());
    let payload = to_put_payload(data);
    let part: RsPartId = self
      .inner
      .put_part(&location, &id, part_idx as usize, payload)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PartId {
      content_id: part.content_id,
    })
  }

  /// Completes a multipart upload.
  ///
  /// `parts[i]` must be the `PartId` returned by `put_part` with `part_idx == i`.
  #[napi]
  pub async fn complete_multipart(
    &self,
    path: String,
    id: String,
    parts: Vec<PartId>,
  ) -> napi::Result<PutResult> {
    let location = Path::from(path.as_str());
    let rs_parts: Vec<RsPartId> = parts
      .into_iter()
      .map(|p| RsPartId {
        content_id: p.content_id,
      })
      .collect();
    let res = self
      .inner
      .complete_multipart(&location, &id, rs_parts)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  /// Aborts a multipart upload.
  #[napi]
  pub async fn abort_multipart(&self, path: String, id: String) -> napi::Result<()> {
    let location = Path::from(path.as_str());
    self
      .inner
      .abort_multipart(&location, &id)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}
