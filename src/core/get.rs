use crate::store::ObjectStore;
use crate::types::{
  build_get_options, convert_attributes, convert_meta, GetOptionsInput, GetResult, Range,
};
use bytes::Bytes;
use http::header::{HeaderMap, HeaderValue, USER_AGENT};
use http::uri::Scheme;
use napi::bindgen_prelude::FromNapiValue;
use napi::bindgen_prelude::{Buffer, Function, Promise, Unknown};
use napi::JsValue;
use napi_derive::napi;
use object_store::client::{
  HttpClient, HttpConnector, HttpError, HttpErrorKind, HttpRequest, HttpResponse,
  HttpResponseBody, HttpService,
};
use object_store::path::Path;
use object_store::{ClientConfigKey, ClientOptions, ObjectStoreExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// The request descriptor handed to the JavaScript fetch adapter.
///
/// `headers` is a plain object; `body` is `null` for bodiless requests
/// (GET/HEAD/DELETE/PROPFIND) and a `Uint8Array` for PUT/COPY payloads.
#[napi(object)]
pub struct FetchRequest {
  pub url: String,
  pub method: String,
  pub headers: HashMap<String, String>,
  pub body: Option<Buffer>,
}

/// The response descriptor the JavaScript fetch adapter must resolve with.
#[napi(object)]
pub struct FetchResponse {
  pub status: u32,
  pub headers: HashMap<String, String>,
  pub body: Buffer,
}

/// The threadsafe wrapper around the JS fetch adapter.
///
/// `CalleeHandled = false` so the adapter is invoked with exactly one
/// argument (the `FetchRequest`), not in Node error-first callback style.
type FetchTsfn = napi::threadsafe_function::ThreadsafeFunction<
  FetchRequest,
  Promise<FetchResponse>,
  FetchRequest,
  napi::Status,
  false,
  false,
  0,
>;

/// Wrap a JavaScript `(request) => Promise<response>` adapter into an
/// [`HttpConnector`] that can be handed to `HttpBuilder::with_http_connector`.
///
/// The adapter runs on Node's event loop and is expected to perform the HTTP
/// request itself — typically by calling the global `fetch` — and to resolve
/// with a [`FetchResponse`] once the full body has been read.
pub(crate) fn fetch_connector(fetch: Unknown) -> napi::Result<NodeFetchConnector> {
  let value = fetch.value();
  let function: Function<'_, FetchRequest, Promise<FetchResponse>> =
    unsafe { Function::from_napi_value(value.env, value.value) }
      .map_err(|e| napi::Error::from_reason(format!("fetch adapter must be a function: {e}")))?;

  let tsfn = function
    .build_threadsafe_function::<FetchRequest>()
    .build()
    .map_err(|e| napi::Error::from_reason(format!("failed to wrap fetch adapter: {e}")))?;

  Ok(NodeFetchConnector {
    fetch: Arc::new(tsfn),
  })
}

/// An [`HttpConnector`] backed by a JavaScript fetch adapter.
///
/// This replaces the `reqwest`-based connector: no Rust HTTP stack is
/// involved, all I/O runs through the host's `fetch` (undici in Node), so
/// proxies, TLS and DNS all follow the runtime's configuration.
#[derive(Clone)]
pub(crate) struct NodeFetchConnector {
  fetch: Arc<FetchTsfn>,
}

impl std::fmt::Debug for NodeFetchConnector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NodeFetchConnector").finish()
  }
}

impl NodeFetchConnector {
  /// Build the [`HttpService`] that this connector drives.
  ///
  /// The [`ClientOptions`] are captured here (rather than at call time) because
  /// [`HttpConnector::connect`] is the only place object_store hands them to a
  /// custom transport. `Clone` is required by
  /// [`SpawnService`](object_store::client::SpawnService).
  pub(crate) fn service(&self, options: &ClientOptions) -> NodeFetchService {
    NodeFetchService {
      fetch: Arc::clone(&self.fetch),
      options: Arc::new(options.clone()),
    }
  }
}

impl HttpConnector for NodeFetchConnector {
  fn connect(&self, options: &ClientOptions) -> object_store::Result<HttpClient> {
    Ok(HttpClient::new(self.service(options)))
  }
}

/// The [`HttpService`] bridging object_store requests to the JS adapter.
#[derive(Clone)]
pub(crate) struct NodeFetchService {
  fetch: Arc<FetchTsfn>,
  options: Arc<ClientOptions>,
}

impl std::fmt::Debug for NodeFetchService {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NodeFetchService").finish()
  }
}

fn fetch_error(e: napi::Error) -> HttpError {
  HttpError::new_boxed(HttpErrorKind::Request, Box::new(e))
}

/// Interpret a `ClientOptions` boolean the same way object_store's own
/// `ConfigValue<bool>` parser does.
fn config_bool(options: &ClientOptions, key: ClientConfigKey) -> bool {
  options
    .get_config_value(&key)
    .is_some_and(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "on" | "yes" | "y"))
}

/// The subset of [`ClientOptions`] that a JavaScript-fetch transport can honour.
///
/// Everything else object_store accepts (`allow_invalid_certificates`,
/// `disable_system_certificates`, `proxy_*`, `pool_*`, `http2_*`, the fine grained
/// timeouts) is consumed exclusively by the built-in `reqwest` transport, which this
/// build does not enable — TLS, proxying, DNS and connection pooling are all
/// whatever the host runtime's `fetch` is configured with.
fn fetch_transport_headers(options: &ClientOptions) -> Result<HeaderMap, HttpError> {
  let mut headers = HeaderMap::new();

  if let Some(defaults) = options.get_default_headers() {
    for (name, value) in defaults.iter() {
      headers.append(name.clone(), value.clone());
    }
  }

  if let Some(agent) = options.get_config_value(&ClientConfigKey::UserAgent) {
    let value = HeaderValue::from_str(&agent).map_err(|e| {
      HttpError::new(HttpErrorKind::Unknown, std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
    })?;
    headers.insert(USER_AGENT, value);
  }

  Ok(headers)
}

/// The overall request timeout, if one is configured.
fn fetch_transport_timeout(options: &ClientOptions) -> Option<Duration> {
  options
    .get_config_value(&ClientConfigKey::Timeout)
    .and_then(|v| humantime::parse_duration(&v).ok())
}

#[async_trait::async_trait]
impl HttpService for NodeFetchService {
  async fn call(&self, req: HttpRequest) -> Result<HttpResponse, HttpError> {
    let (mut parts, body) = req.into_parts();

    if !config_bool(&self.options, ClientConfigKey::AllowHttp)
      && parts.uri.scheme() == Some(&Scheme::HTTP)
    {
      return Err(HttpError::new(
        HttpErrorKind::Unknown,
        std::io::Error::new(
          std::io::ErrorKind::PermissionDenied,
          format!(
            "refusing to use the non-TLS URI {}: call \
             `ClientOptions::withAllowHttp(true)` to permit plain http",
            parts.uri
          ),
        ),
      ));
    }

    // Default headers only fill gaps; an explicit per-request header wins.
    for (name, value) in fetch_transport_headers(&self.options)?.iter() {
      if !parts.headers.contains_key(name) {
        parts.headers.append(name.clone(), value.clone());
      }
    }

    let url = parts.uri.to_string();
    let method = parts.method.as_str().to_string();
    let headers: HashMap<String, String> = parts
      .headers
      .iter()
      .map(|(k, v)| {
        (
          k.as_str().to_string(),
          v.to_str().unwrap_or_default().to_string(),
        )
      })
      .collect();

    // Buffer the request body (object_store bodies are in-memory payloads).
    let body_bytes: Bytes = if body.content_length() == 0 {
      Bytes::new()
    } else {
      use http_body_util::BodyExt;
      body.collect().await?.to_bytes()
    };

    let request = FetchRequest {
      url,
      method,
      headers,
      body: if body_bytes.is_empty() {
        None
      } else {
        Some(Buffer::from(body_bytes.to_vec()))
      },
    };

    let call = async {
      // Hop to the JS thread: call the adapter, then await the resolved promise.
      let response: FetchResponse = self
        .fetch
        .call_async(request)
        .await
        .map_err(fetch_error)?
        .await
        .map_err(fetch_error)?;

      let status = u16::try_from(response.status).map_err(|_| {
        HttpError::new_boxed(
          HttpErrorKind::Unknown,
          Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid response status {}", response.status),
          )),
        )
      })?;

      let mut builder = http::Response::builder().status(status);
      for (name, value) in &response.headers {
        builder = builder.header(name.as_str(), value.as_str());
      }
      builder
        .body(HttpResponseBody::from(response.body.to_vec()))
        .map_err(|e| HttpError::new_boxed(HttpErrorKind::Unknown, Box::new(e)))
    };

    match fetch_transport_timeout(&self.options) {
      // `Timeout` is retried by object_store when the request is idempotent.
      Some(limit) => tokio::time::timeout(limit, call)
        .await
        .unwrap_or_else(|_| {
          Err(HttpError::new(
            HttpErrorKind::Timeout,
            std::io::Error::new(std::io::ErrorKind::TimedOut, "request timed out"),
          ))
        }),
      None => call.await,
    }
  }
}

#[napi]
impl ObjectStore {
  /// Return the bytes stored at `path`.
  ///
  /// Wraps `ObjectStoreExt::get`. Passing `options` routes through
  /// `ObjectStore::get_opts` (conditional gets).
  #[napi]
  pub async fn get(
    &self,
    path: String,
    options: Option<GetOptionsInput>,
  ) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());

    if let Some(opts_input) = options {
      let opts = build_get_options(&opts_input);
      let res = self
        .inner
        .get_opts(&location, opts)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      let bytes = res
        .bytes()
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      return Ok(Buffer::from(bytes.as_ref()));
    }

    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }

  /// Perform a get request with options
  /// (conditionals, ranges, versioning via `ObjectStore::get_opts`).
  #[napi]
  pub async fn get_opts(&self, path: String, options: GetOptionsInput) -> napi::Result<Buffer> {
    self.get(path, Some(options)).await
  }

  /// Return the bytes and metadata stored at `path`.
  #[napi]
  pub async fn get_with_meta(&self, path: String) -> napi::Result<GetResult> {
    let location = Path::from(path.as_str());
    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    let meta = convert_meta(&res.meta);
    let range = Range {
      start: res.range.start as f64,
      end: res.range.end as f64,
    };
    let attributes = convert_attributes(&res.attributes);
    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(GetResult {
      bytes: Buffer::from(bytes.as_ref()),
      meta,
      range,
      attributes,
    })
  }

  /// Return the bytes stored at `path` within `range` (`{ start, end }`,
  /// end exclusive).
  ///
  /// Wraps `ObjectStoreExt::get_range`.
  #[napi]
  pub async fn get_range(&self, path: String, range: Range) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
    let bytes = self
      .inner
      .get_range(&location, (range.start as u64)..(range.end as u64))
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }
}
