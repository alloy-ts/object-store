use crate::core::store::ObjectStore;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn delete(&self, path: String) -> napi::Result<()> {
    let location = Path::from(path.as_str());
    self
      .inner
      .delete(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}
