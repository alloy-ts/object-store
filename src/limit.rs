use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::limit::LimitStore as RSLimitStore;
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;

/// NAPI binding for `object_store::limit::LimitStore`.
///
/// Wraps any [`ObjectStore`] and bounds the number of concurrent outstanding
/// operations (each `ObjectStore` member call counts as one operation, even if it
/// issues multiple network requests).
///
/// The wrapped store is itself an [`ObjectStore`], so [`LimitStore::as_object_store`]
/// hands back the existing [`NapiObjectStore`] for the full put/get/list/... surface.
///
/// The inner store is held as `Arc<dyn ObjectStore>` (a sized type that implements
/// [`ObjectStore`]), which satisfies `LimitStore<T>`'s `Sized` bound on `T`.
#[napi]
pub struct LimitStore {
  pub(crate) inner: Arc<RSLimitStore<Arc<dyn ObjectStoreTrait>>>,
}

#[napi]
impl LimitStore {
  /// Wrap `store` so that at most `max_requests` operations are in flight at once.
  ///
  /// `max_requests` must be at least 1; a zero limit would deadlock on the first
  /// operation (a semaphore with zero permits never admits a caller).
  #[napi(factory)]
  pub fn new(store: &NapiObjectStore, max_requests: u32) -> napi::Result<Self> {
    if max_requests == 0 {
      return Err(napi::Error::from_reason(
        "max_requests must be >= 1".to_string(),
      ));
    }
    let limited = RSLimitStore::new(store.inner.clone(), max_requests as usize);
    Ok(Self {
      inner: Arc::new(limited),
    })
  }

  /// Return the underlying limited store as a regular `ObjectStore` so the full
  /// put/get/list/... surface can be used (now concurrency-limited).
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }
}
