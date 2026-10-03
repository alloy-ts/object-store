use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::ObjectStore;

pub async fn store_rename(store: &ObjectStore, from: String, to: String) -> Result<()> {
  let from_p = Path::from(from.as_str());
  let to_p = Path::from(to.as_str());
  store
    .inner
    .rename(&from_p, &to_p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
