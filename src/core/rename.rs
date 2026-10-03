use crate::store::ObjectStore;
use crate::types::RenameOptionsInput;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  /// Move an object from `from` to `to`, overwriting any existing object.
  ///
  /// Wraps `ObjectStoreExt::rename` (copy + delete source). Passing
  /// `targetModeCreate` routes through `ObjectStoreExt::rename_if_not_exists`.
  #[napi]
  pub async fn rename(
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

  /// Move an object from `from` to `to` with options
  /// (`ObjectStore::rename_opts`).
  #[napi]
  pub async fn rename_opts(
    &self,
    from: String,
    to: String,
    options: Option<RenameOptionsInput>,
  ) -> napi::Result<()> {
    self.rename(from, to, options).await
  }

  /// Move an object only if the destination does not already exist.
  ///
  /// Wraps `ObjectStoreExt::rename_if_not_exists`; errors with
  /// `AlreadyExists` when the destination is populated.
  #[napi]
  pub async fn rename_if_not_exists(&self, from: String, to: String) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .rename_if_not_exists(&from_path, &to_path)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }
}
