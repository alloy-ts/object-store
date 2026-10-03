use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::registry::{DefaultObjectStoreRegistry, ObjectStoreRegistry as _};
use std::sync::Arc;
use url::Url;

/// NAPI binding for [`object_store::registry::DefaultObjectStoreRegistry`].
///
/// Maps object URLs to [`ObjectStore`] instances. A store registered with
/// [`register`](Self::register) (or, for understood schemes such as `file://`,
/// `memory://` and `http(s)://`, lazily created by `resolve`) is returned from
/// [`resolve`](Self::resolve) together with the relative path suffix. Longest
/// registered path-prefix wins.
///
/// `resolve` returns a `[store, path]` tuple: the matched store and the trailing
/// path (a [`object_store::path::Path`] rendered as a string) relative to the
/// registered base URL.
#[napi]
pub struct ObjectStoreRegistry {
  inner: Arc<DefaultObjectStoreRegistry>,
}

#[napi]
impl ObjectStoreRegistry {
  /// Create a new, empty registry.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Arc::new(DefaultObjectStoreRegistry::new()),
    }
  }

  /// Register `store` for `url`, replacing any store previously registered at
  /// exactly that URL. Returns the previous store, if one existed.
  #[napi]
  pub fn register(
    &self,
    url: String,
    store: &NapiObjectStore,
  ) -> napi::Result<Option<NapiObjectStore>> {
    let parsed =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let previous = self.inner.register(parsed, store.inner.clone());
    Ok(previous.map(|s| NapiObjectStore { inner: s }))
  }

  /// Resolve `url` to a registered (or lazily created) store and the trailing
  /// path. The longest registered path prefix is matched; if none matches, the
  /// registry may create a store based on the URL (e.g. `file://`, `memory://`).
  ///
  /// Returns a `[store, path]` tuple. Errors if no store matches and none can be
  /// created for the URL's scheme.
  #[napi]
  pub fn resolve(&self, url: String) -> napi::Result<(NapiObjectStore, String)> {
    let parsed =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let (store, path) = self
      .inner
      .resolve(&parsed)
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok((NapiObjectStore { inner: store }, path.to_string()))
  }

  /// Remove the store registered at exactly `url`, returning it if present.
  ///
  /// This is the inverse of [`register`](Self::register): it removes only the
  /// store at the exact scheme/authority/path, so a store registered at a
  /// different path under the same authority is left in place.
  #[napi]
  pub fn deregister(&self, url: String) -> napi::Result<Option<NapiObjectStore>> {
    let parsed =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let removed = self.inner.deregister(&parsed);
    Ok(removed.map(|s| NapiObjectStore { inner: s }))
  }
}
