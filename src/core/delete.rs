use futures::StreamExt;
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

pub async fn store_delete_stream(store: &ObjectStore, paths: Vec<String>) -> Result<()> {
  let stream_locations =
    futures::stream::iter(paths.into_iter().map(|s| Ok(Path::from(s.as_str())))).boxed();
  let mut res_stream = store.inner.delete_stream(stream_locations);
  while let Some(res) = res_stream.next().await {
    res.map_err(|e| Error::from_reason(e.to_string()))?;
  }
  Ok(())
}
