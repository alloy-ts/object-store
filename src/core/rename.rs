use crate::core::store::ObjectStore;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn rename(&self, from: String, to: String) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .rename(&from_path, &to_path)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}
