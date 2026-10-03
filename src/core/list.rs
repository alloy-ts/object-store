use crate::core::store::ObjectStore;
use crate::core::types::{convert_meta, ListOptionsInput, ListResult, ObjectMeta};
use futures::StreamExt;
use napi_derive::napi;
use object_store::path::Path;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn list(
    &self,
    prefix: Option<String>,
    _options: Option<ListOptionsInput>,
  ) -> napi::Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let mut stream = self.inner.list(prefix_path.as_ref());
    let mut results = Vec::new();
    while let Some(item) = stream.next().await {
      let meta = item.map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      results.push(convert_meta(&meta));
    }
    Ok(results)
  }

  #[napi]
  pub async fn list_opts(
    &self,
    prefix: Option<String>,
    options: Option<ListOptionsInput>,
  ) -> napi::Result<Vec<ObjectMeta>> {
    self.list(prefix, options).await
  }

  #[napi]
  pub async fn list_with_delimiter(&self, prefix: Option<String>) -> napi::Result<ListResult> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let res = self
      .inner
      .list_with_delimiter(prefix_path.as_ref())
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(ListResult {
      objects: res.objects.iter().map(convert_meta).collect(),
      common_prefixes: res.common_prefixes.iter().map(|p| p.to_string()).collect(),
    })
  }
}
