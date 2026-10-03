use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::ObjectStore;

pub async fn delete(store: &ObjectStore, path: String) -> Result<()> {
  let location = Path::from(path.as_str());
  store
    .inner
    .delete(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
