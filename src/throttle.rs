use crate::memory::InMemory as NapiInMemory;
use crate::multipart::MultipartStore as NapiMultipartStore;
use crate::store::ObjectStore as NapiObjectStore;
use napi_derive::napi;
use object_store::memory::InMemory as RSInMemory;
use object_store::multipart::MultipartStore as MultipartStoreTrait;
use object_store::throttle::{ThrottledStore as RSThrottledStore, ThrottleConfig as RsThrottleConfig};
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;
use std::time::Duration;

/// Throttling configuration for a [`ThrottledStore`], expressed in milliseconds.
///
/// Every field is optional; unspecified waits default to zero. Mirrors
/// `object_store::throttle::ThrottleConfig`.
#[napi(object)]
pub struct ThrottleConfig {
  pub wait_delete_per_call: Option<f64>,
  pub wait_get_per_byte: Option<f64>,
  pub wait_get_per_call: Option<f64>,
  pub wait_list_per_call: Option<f64>,
  pub wait_list_per_entry: Option<f64>,
  pub wait_list_with_delimiter_per_call: Option<f64>,
  pub wait_list_with_delimiter_per_entry: Option<f64>,
  pub wait_put_per_call: Option<f64>,
}

fn to_throttle_config(input: &ThrottleConfig) -> RsThrottleConfig {
  let ms = |v: Option<f64>| Duration::from_secs_f64(v.unwrap_or(0.0) / 1000.0);
  RsThrottleConfig {
    wait_delete_per_call: ms(input.wait_delete_per_call),
    wait_get_per_byte: ms(input.wait_get_per_byte),
    wait_get_per_call: ms(input.wait_get_per_call),
    wait_list_per_call: ms(input.wait_list_per_call),
    wait_list_per_entry: ms(input.wait_list_per_entry),
    wait_list_with_delimiter_per_call: ms(input.wait_list_with_delimiter_per_call),
    wait_list_with_delimiter_per_entry: ms(input.wait_list_with_delimiter_per_entry),
    wait_put_per_call: ms(input.wait_put_per_call),
  }
}

fn from_throttle_config(cfg: &RsThrottleConfig) -> ThrottleConfig {
  let ms = |d: Duration| d.as_secs_f64() * 1000.0;
  ThrottleConfig {
    wait_delete_per_call: Some(ms(cfg.wait_delete_per_call)),
    wait_get_per_byte: Some(ms(cfg.wait_get_per_byte)),
    wait_get_per_call: Some(ms(cfg.wait_get_per_call)),
    wait_list_per_call: Some(ms(cfg.wait_list_per_call)),
    wait_list_per_entry: Some(ms(cfg.wait_list_per_entry)),
    wait_list_with_delimiter_per_call: Some(ms(cfg.wait_list_with_delimiter_per_call)),
    wait_list_with_delimiter_per_entry: Some(ms(cfg.wait_list_with_delimiter_per_entry)),
    wait_put_per_call: Some(ms(cfg.wait_put_per_call)),
  }
}

/// NAPI binding for `object_store::throttle::ThrottledStore`.
///
/// Wraps an [`InMemory`] store with deterministic `sleep` calls for performance
/// testing. Because `ThrottledStore<InMemory>` implements both `ObjectStore` and
/// `MultipartStore`, this class exposes the throttle config and hands back the
/// existing [`NapiObjectStore`] / [`NapiMultipartStore`] bindings for the actual
/// read/write operations (no operation methods are re-implemented here).
#[napi]
pub struct ThrottledStore {
  pub(crate) inner: Arc<RSThrottledStore<RSInMemory>>,
}

#[napi]
impl ThrottledStore {
  /// Create a throttled wrapper around an in-memory store.
  ///
  /// `config` is optional; omitted waits default to zero (i.e. no throttling).
  #[napi(factory)]
  pub fn new(inner: &NapiInMemory, config: Option<ThrottleConfig>) -> Self {
    let cfg = config.map(|c| to_throttle_config(&c)).unwrap_or_default();
    // Use the *real* `object_store::memory::InMemory` (held by the NAPI wrapper)
    // as the generic `T`, since only that type implements `ObjectStore` /
    // `MultipartStore`.
    let ts = RSThrottledStore::new((*inner.inner).clone(), cfg);
    Self {
      inner: Arc::new(ts),
    }
  }

  /// Return the underlying store as a regular `ObjectStore` so the full
  /// put/get/list/... surface can be used (now throttled).
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }

  /// Return the underlying store as a `MultipartStore` for the low-level
  /// multipart API (now throttled).
  #[napi]
  pub fn as_multipart_store(&self) -> NapiMultipartStore {
    let inner: Arc<dyn MultipartStoreTrait> = self.inner.clone();
    NapiMultipartStore { inner }
  }

  /// Return a copy of the current throttle configuration.
  #[napi]
  pub fn get_config(&self) -> ThrottleConfig {
    from_throttle_config(&self.inner.config())
  }

  /// Replace the entire throttle configuration.
  #[napi]
  pub fn configure(&self, config: ThrottleConfig) {
    let cfg = to_throttle_config(&config);
    self.inner.config_mut(|c| *c = cfg);
  }
}
