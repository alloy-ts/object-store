use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::chunked::ChunkedStore as RSChunkedStore;
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;

/// NAPI binding for `object_store::chunked::ChunkedStore`.
///
/// Wraps any [`ObjectStore`] and forces its `get` responses to be returned in
/// fixed-size chunks. Intended for tests that need to control stream chunking
/// (e.g. verifying delimiter logic in a newline-delimited stream).
///
/// The wrapped store is itself an [`ObjectStore`], so [`ChunkedStore::as_object_store`]
/// hands back the existing [`NapiObjectStore`] for the full put/get/list/... surface.
#[napi]
pub struct ChunkedStore {
  pub(crate) inner: Arc<dyn ObjectStoreTrait>,
}

#[napi]
impl ChunkedStore {
  /// Wrap `store` so that `get` responses return chunks of at most `chunk_size` bytes.
  ///
  /// `chunk_size` must be at least 1; a zero chunk size would otherwise yield an
  /// infinite stream of empty chunks.
  #[napi(factory)]
  pub fn new(store: &NapiObjectStore, chunk_size: u32) -> napi::Result<Self> {
    if chunk_size == 0 {
      return Err(napi::Error::from_reason("chunk_size must be >= 1".to_string()));
    }
    let chunked = RSChunkedStore::new(store.inner.clone(), chunk_size as usize);
    Ok(Self {
      inner: Arc::new(chunked),
    })
  }

  /// Return the underlying chunked store as a regular `ObjectStore` so the full
  /// put/get/list/... surface can be used (now returning chunked streams).
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }
}
