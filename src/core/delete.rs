use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::ObjectStore;

#[napi(object)]
pub struct DeleteOptions {}

pub async fn delete(store: &ObjectStore, path: String) -> Result<()> {
  let location = Path::from(path.as_str());
  store
    .inner
    .delete(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}

pub async fn delete_opts(
  store: &ObjectStore,
  path: String,
  _options: Option<DeleteOptions>,
) -> Result<()> {
  delete(store, path).await
}
