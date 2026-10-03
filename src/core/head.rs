use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::{convert_meta, ObjectMeta, ObjectStore};

pub async fn store_head(store: &ObjectStore, path: String) -> Result<ObjectMeta> {
  let p = Path::from(path.as_str());
  let meta = store
    .inner
    .head(&p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(convert_meta(&meta))
}
