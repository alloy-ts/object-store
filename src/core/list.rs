use futures::TryStreamExt;
use napi::bindgen_prelude::*;
use object_store::path::Path;

use crate::core::types::{convert_meta, ListResult, ObjectMeta, ObjectStore};

pub async fn store_list(store: &ObjectStore, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
  let p = prefix.map(|s| Path::from(s.as_str()));
  let stream = store.inner.list(p.as_ref());
  let metas: Vec<object_store::ObjectMeta> = stream
    .try_collect()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(metas.iter().map(convert_meta).collect())
}

pub async fn store_list_with_delimiter(
  store: &ObjectStore,
  prefix: Option<String>,
) -> Result<ListResult> {
  let p = prefix.map(|s| Path::from(s.as_str()));
  let res = store
    .inner
    .list_with_delimiter(p.as_ref())
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let objects = res.objects.iter().map(convert_meta).collect();
  let common_prefixes = res
    .common_prefixes
    .iter()
    .map(|cp| cp.to_string())
    .collect();
  Ok(ListResult {
    objects,
    common_prefixes,
  })
}
