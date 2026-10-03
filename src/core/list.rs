use futures::StreamExt;
use napi::bindgen_prelude::*;
use object_store::path::Path;

use super::types::{ObjectMeta, ObjectStore};

pub async fn list(store: &ObjectStore, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
  let prefix_path = prefix.map(|p| Path::from(p.as_str()));
  let mut stream = store.inner.list(prefix_path.as_ref());

  let mut result = Vec::new();
  while let Some(meta_res) = stream.next().await {
    let meta = meta_res.map_err(|e| Error::from_reason(e.to_string()))?;
    result.push(ObjectMeta::from(meta));
  }

  Ok(result)
}
