use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::local::LocalFileSystem as RSFileSystem;
use object_store::path::Path;
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;

/// NAPI binding for `object_store::local::LocalFileSystem`.
///
/// Exposes the filesystem-specific constructors and tuning methods, and hands back
/// the existing [`NapiObjectStore`] binding for the actual put/get/list/... surface
/// (no operation methods are re-implemented here).
#[napi]
pub struct LocalFileSystem {
  pub(crate) inner: Arc<RSFileSystem>,
}

#[napi]
impl LocalFileSystem {
  /// Create new filesystem storage with no prefix (maps to the filesystem root).
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Arc::new(RSFileSystem::new()),
    }
  }

  /// Create new filesystem storage with `prefix` applied to all paths.
  ///
  /// Returns an error if the prefix path does not exist.
  #[napi(factory)]
  pub fn new_with_prefix(prefix: String) -> napi::Result<LocalFileSystem> {
    let fs = RSFileSystem::new_with_prefix(&prefix)
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(fs),
    })
  }

  /// Return an absolute filesystem path of the given object location.
  #[napi]
  pub fn path_to_filesystem(&self, location: String) -> napi::Result<String> {
    let p = Path::from(location.as_str());
    let path = self
      .inner
      .path_to_filesystem(&p)
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(path.to_string_lossy().into_owned())
  }

  /// Enable automatic cleanup of empty directories when deleting files.
  ///
  /// Returns a new store with the setting applied (the builder does not mutate
  /// in place).
  #[napi]
  pub fn with_automatic_cleanup(&self, automatic_cleanup: bool) -> LocalFileSystem {
    let fs = self
      .inner
      .as_ref()
      .clone()
      .with_automatic_cleanup(automatic_cleanup);
    Self {
      inner: Arc::new(fs),
    }
  }

  /// Enable `fsync` after writes for durability (disabled by default).
  ///
  /// Returns a new store with the setting applied (the builder does not mutate
  /// in place).
  #[napi]
  pub fn with_fsync(&self, fsync: bool) -> LocalFileSystem {
    let fs = self.inner.as_ref().clone().with_fsync(fsync);
    Self {
      inner: Arc::new(fs),
    }
  }

  /// Return the underlying store as a regular `ObjectStore` so the full
  /// put/get/list/... surface can be used.
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }
}
