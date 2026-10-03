use crate::store::ObjectStore;
use crate::types::{convert_meta, HeadOptionsInput, ObjectMeta};
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn head(
    &self,
    path: String,
    _options: Option<HeadOptionsInput>,
  ) -> napi::Result<ObjectMeta> {
    let location = Path::from(path.as_str());
    let meta = self
      .inner
      .head(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(convert_meta(&meta))
  }

  #[napi]
  pub async fn head_opts(
    &self,
    path: String,
    options: Option<HeadOptionsInput>,
  ) -> napi::Result<ObjectMeta> {
    self.head(path, options).await
  }
}
