use crate::core::store::ObjectStore;
use crate::core::types::RenameOptionsInput;
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

  #[napi]
  pub async fn rename_opts(
    &self,
    from: String,
    to: String,
    options: Option<RenameOptionsInput>,
  ) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    if options.and_then(|o| o.target_mode_create) == Some(true) {
      self
        .inner
        .rename_if_not_exists(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    } else {
      self
        .inner
        .rename(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    }
    Ok(())
  }
}
