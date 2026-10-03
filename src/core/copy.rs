use crate::store::ObjectStore;
use crate::types::CopyOptionsInput;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn copy(
    &self,
    from: String,
    to: String,
    options: Option<CopyOptionsInput>,
  ) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    if options.and_then(|o| o.if_not_exists) == Some(true) {
      self
        .inner
        .copy_if_not_exists(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    } else {
      self
        .inner
        .copy(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    }
    Ok(())
  }

  #[napi]
  pub async fn copy_opts(
    &self,
    from: String,
    to: String,
    options: Option<CopyOptionsInput>,
  ) -> napi::Result<()> {
    self.copy(from, to, options).await
  }
  #[napi]
  pub async fn copy_if_not_exists(&self, from: String, to: String) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .copy_if_not_exists(&from_path, &to_path)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}
