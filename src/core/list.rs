use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;

use super::types::{ObjectMeta, ObjectStore};

#[napi(object)]
pub struct ListResult {
  pub objects: Vec<ObjectMeta>,
  pub prefixes: Vec<String>,
}

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

pub async fn list_with_offset(
  store: &ObjectStore,
  prefix: Option<String>,
  offset: String,
) -> Result<Vec<ObjectMeta>> {
  let prefix_path = prefix.map(|p| Path::from(p.as_str()));
  let offset_path = Path::from(offset.as_str());
  let mut stream = store.inner.list_with_offset(prefix_path.as_ref(), &offset_path);

  let mut result = Vec::new();
  while let Some(meta_res) = stream.next().await {
    let meta = meta_res.map_err(|e| Error::from_reason(e.to_string()))?;
    result.push(ObjectMeta::from(meta));
  }

  Ok(result)
}
