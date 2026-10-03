use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::{ObjectMeta, ObjectStore};

#[napi(object)]
pub struct GetResult {
  pub meta: ObjectMeta,
  pub bytes: Buffer,
}

pub async fn get(store: &ObjectStore, path: String) -> Result<Buffer> {
  let location = Path::from(path.as_str());
  let res = store
    .inner
    .get(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let bytes = res
    .bytes()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(Buffer::from(bytes.as_ref()))
}

pub async fn get_result(store: &ObjectStore, path: String) -> Result<GetResult> {
  let location = Path::from(path.as_str());
  let res = store
    .inner
    .get(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let meta = ObjectMeta::from(res.meta.clone());
  let bytes = res
    .bytes()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(GetResult {
    meta,
    bytes: Buffer::from(bytes.as_ref()),
  })
}
