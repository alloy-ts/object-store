use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::ObjectStore;

pub async fn copy(store: &ObjectStore, from: String, to: String) -> Result<()> {
  let from_path = Path::from(from.as_str());
  let to_path = Path::from(to.as_str());
  store
    .inner
    .copy(&from_path, &to_path)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
