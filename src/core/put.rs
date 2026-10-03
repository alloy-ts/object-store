use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::{ObjectStoreExt, PutPayload};

use super::put_opts::PutResult;
use super::types::ObjectStore;

pub async fn put(store: &ObjectStore, path: String, payload: Uint8Array) -> Result<PutResult> {
  let location = Path::from(path.as_str());
  let put_payload = PutPayload::from(payload.to_vec());
  let res = store
    .inner
    .put(&location, put_payload)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}
