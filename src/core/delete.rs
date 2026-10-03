use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::ObjectStore;

pub async fn store_delete(store: &ObjectStore, path: String) -> Result<()> {
  let p = Path::from(path.as_str());
  store
    .inner
    .delete(&p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
