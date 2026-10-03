use crate::core::store::ObjectStore;
use crate::core::types::{PutOptionsInput, PutResult};
use bytes::Bytes;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{ObjectStoreExt, PutMode, PutOptions, PutPayload, UpdateVersion};

#[napi]
impl ObjectStore {
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
      let mode = if opts.mode_create == Some(true) {
        PutMode::Create
      } else if let Some(update_ver) = opts.mode_update {
        PutMode::Update(UpdateVersion {
          e_tag: update_ver.e_tag,
          version: update_ver.version,
        })
      } else {
        PutMode::Overwrite
      };

      let put_options = PutOptions::from(mode);
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

  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    data: Buffer,
    options: PutOptionsInput,
  ) -> napi::Result<PutResult> {
    self.put(path, data, Some(options)).await
  }
}
