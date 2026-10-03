use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::ObjectStore;

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
