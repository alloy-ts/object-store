use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use super::types::{ObjectMeta, ObjectStore};

pub async fn head(store: &ObjectStore, path: String) -> Result<ObjectMeta> {
  let location = Path::from(path.as_str());
  let meta = store
    .inner
    .head(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ObjectMeta::from(meta))
}
