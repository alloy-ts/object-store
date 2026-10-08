use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::memory::InMemory as RSInMemory;
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;

/// In-memory storage suitable for testing or for opting out of using a cloud
/// storage provider.
///
/// Wraps `object_store::memory::InMemory`. Unlike the generic [`crate::store::ObjectStore`]
/// (which erases the concrete type behind `dyn ObjectStore`), this binding keeps the
/// concrete `InMemory` type so it can expose [`InMemory::fork`], which snapshots the
/// current contents into a brand-new store, and so it can be passed to
/// [`crate::multipart::MultipartStore::from_in_memory`] and
/// [`crate::throttle::ThrottledStore::new`].
///
/// All object operations (put/get/head/list/delete/copy/rename/...) live on the
/// core `ObjectStore` API; use [`InMemory::as_object_store`] to get one backed by
/// this store.
#[napi]
pub struct InMemory {
  pub(crate) inner: Arc<RSInMemory>,
}

#[napi]
impl InMemory {
  /// Create new in-memory storage.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Arc::new(RSInMemory::new()),
    }
  }

  /// Creates a fork of the store, with the current content copied into the new store.
  #[napi]
  pub fn fork(&self) -> InMemory {
    let forked = self.inner.fork();
    InMemory {
      inner: Arc::new(forked),
    }
  }

  /// Return this store as a regular `ObjectStore` so the full core
  /// put/get/list/... surface can be used against the same backing storage.
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }
}
