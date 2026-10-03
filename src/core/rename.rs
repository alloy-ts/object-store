use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::ObjectStore;

#[napi(object)]
pub struct RenameOptions {
  #[napi(ts_type = "'overwrite' | 'create'")]
  pub target_mode: Option<String>,
}

pub async fn rename(store: &ObjectStore, from: String, to: String) -> Result<()> {
  let from_path = Path::from(from.as_str());
  let to_path = Path::from(to.as_str());
  store
    .inner
    .rename(&from_path, &to_path)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}

pub async fn rename_opts(
  store: &ObjectStore,
  from: String,
  to: String,
  _options: Option<RenameOptions>,
) -> Result<()> {
  rename(store, from, to).await
}
