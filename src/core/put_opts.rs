use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::{PutMode, PutOptions as ObjPutOptions, PutPayload, UpdateVersion};

use super::types::{ObjectStore, PutOptions, PutResult};

pub async fn put_opts(
  store: &ObjectStore,
  path: String,
  payload: Uint8Array,
  options: Option<PutOptions>,
) -> Result<PutResult> {
  let location = Path::from(path.as_str());
  let put_payload = PutPayload::from(payload.to_vec());

  let put_mode = if let Some(opts) = &options {
    match opts.mode.as_deref() {
      Some("create") => PutMode::Create,
      Some("update") => PutMode::Update(UpdateVersion {
        e_tag: opts.e_tag.clone(),
        version: opts.version.clone(),
      }),
      _ => PutMode::Overwrite,
    }
  } else {
    PutMode::Overwrite
  };

  let put_options = ObjPutOptions::from(put_mode);
  let res = store
    .inner
    .put_opts(&location, put_payload, put_options)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;

  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}
