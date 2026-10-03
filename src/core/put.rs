use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::{ObjectStore, PutOptionsParam, PutResult};

pub async fn store_put(store: &ObjectStore, path: String, bytes: Buffer) -> Result<PutResult> {
  let p = Path::from(path.as_str());
  let payload = object_store::PutPayload::from(bytes.to_vec());
  let res = store
    .inner
    .put(&p, payload)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}

pub async fn store_put_opts(
  store: &ObjectStore,
  path: String,
  bytes: Buffer,
  options: PutOptionsParam,
) -> Result<PutResult> {
  let p = Path::from(path.as_str());
  let payload = object_store::PutPayload::from(bytes.to_vec());
  let mut opts = object_store::PutOptions::default();
  if let Some(m) = options.mode {
    if m == "create" {
      opts.mode = object_store::PutMode::Create;
    }
  }
  let res = store
    .inner
    .put_opts(&p, payload, opts)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}
