use crate::core::get::fetch_connector;
use crate::store::ObjectStore as NapiObjectStore;
use napi::bindgen_prelude::Unknown;
use napi_derive::napi;
use object_store::http::{HttpBuilder, HttpStore as RSHttpStore};
use object_store::ClientOptions;
use object_store::ObjectStore as ObjectStoreTrait;
use object_store::RetryConfig;
use std::sync::Arc;

/// Optional configuration for constructing an [`HttpStore`].
///
/// All fields are optional; omitted values fall back to object_store's defaults.
#[napi(object)]
pub struct HttpOptions {
  /// Allow `http://` (non-TLS) URLs. By default only `https://` is permitted.
  pub allow_http: Option<bool>,
  /// Maximum number of times a request is retried before failing.
  pub retry_max_attempts: Option<u32>,
  /// HTTP proxy URL used for all requests.
  pub proxy_url: Option<String>,
}

/// NAPI binding for `object_store::http::HttpStore`.
///
/// An [`ObjectStore`] implementation for generic HTTP/WebDAV servers.
///
/// All HTTP I/O is performed by a **JavaScript fetch adapter** — a
/// `(request) => Promise<response>` function supplied at construction —
/// instead of a Rust HTTP client (reqwest is not enabled in this build).
/// The adapter receives a `FetchRequest` `{ url, method, headers, body }`
/// and must resolve with a `FetchResponse` `{ status, headers, body }`
/// after reading the full response body:
///
/// ```js
/// import { HttpStore } from "./index.js";
///
/// const fetchAdapter = async (request) => {
///   const res = await fetch(request.url, {
///     method: request.method,
///     headers: request.headers,
///     body: request.body ?? undefined,
///   });
///   const body = await res.arrayBuffer();
///   return {
///     status: res.status,
///     headers: Object.fromEntries(res.headers),
///     body: Buffer.from(body),
///   };
/// };
///
/// const store = new HttpStore("https://example.com/root", fetchAdapter);
/// ```
///
/// Use [`HttpStore::as_object_store`] to access the full put/get/list/...
/// surface. Note the response body is fully buffered by the adapter, and
/// `put_multipart` / `rename` return `NotImplemented` (matching
/// object_store's own limitations for HTTP).
#[napi]
pub struct HttpStore {
  pub(crate) inner: Arc<RSHttpStore>,
}

#[napi]
impl HttpStore {
  /// Create a new [`HttpStore`] for the given base `url`.
  ///
  /// `fetch` is the JavaScript fetch adapter described in the class docs;
  /// every request the store makes is delegated to it.
  #[napi(constructor)]
  pub fn new(url: String, fetch: Unknown) -> napi::Result<Self> {
    Self::build(url, fetch, None)
  }

  /// Create a new [`HttpStore`] for the given base `url` with options.
  #[napi(factory)]
  pub fn with_options(url: String, fetch: Unknown, options: HttpOptions) -> napi::Result<Self> {
    Self::build(url, fetch, Some(options))
  }

  /// Create a new [`HttpStore`] for the given base `url` with [`ClientOptions`](crate::client::ClientOptions).
  #[napi(factory)]
  pub fn with_client_options(
    url: String,
    fetch: Unknown,
    client_options: &crate::client::ClientOptions,
  ) -> napi::Result<Self> {
    let connector = fetch_connector(fetch).map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let store = HttpBuilder::new()
      .with_url(url)
      .with_http_connector(connector)
      .with_client_options(client_options.into())
      .build()
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;

    Ok(Self {
      inner: Arc::new(store),
    })
  }

  fn build(url: String, fetch: Unknown, options: Option<HttpOptions>) -> napi::Result<Self> {
    let connector = fetch_connector(fetch).map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut builder = HttpBuilder::new()
      .with_url(url)
      .with_http_connector(connector);

    if let Some(opts) = options {
      let mut client_options = ClientOptions::new();
      if let Some(true) = opts.allow_http {
        client_options = client_options.with_allow_http(true);
      }
      if let Some(proxy) = opts.proxy_url {
        client_options = client_options.with_proxy_url(proxy);
      }
      builder = builder.with_client_options(client_options);

      let mut retry = RetryConfig::default();
      if let Some(attempts) = opts.retry_max_attempts {
        retry.max_retries = attempts as usize;
      }
      builder = builder.with_retry(retry);
    }

    let store = builder
      .build()
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(store),
    })
  }

  /// Return the underlying store as a regular `ObjectStore` so the full
  /// put/get/list/... surface can be used.
  #[napi]
  pub fn as_object_store(&self) -> NapiObjectStore {
    let inner: Arc<dyn ObjectStoreTrait> = self.inner.clone();
    NapiObjectStore { inner }
  }
}
