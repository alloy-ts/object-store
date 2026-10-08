use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::{Method, Uri};
use napi::bindgen_prelude::{Buffer, ClassInstance, FromNapiValue, Unknown};
use napi::sys;
use napi_derive::napi;
use object_store::client::{
  HttpClient as RsHttpClient, HttpConnector, HttpError as RsHttpError, HttpRequest as RsHttpRequest,
  HttpRequestBody,
};
use object_store::path::Path;
use object_store::{ClientConfigKey, ClientOptions as RsClientOptions};

use crate::core::get::fetch_connector;
use crate::spawn::IoRuntime;

/// NAPI binding for `object_store::ClientOptions`.
///
/// This is the configuration object every object_store builder accepts, exposed
/// so it can be assembled once in JavaScript and then shared by every store or
/// [`HttpClient`].
///
/// # Which options actually take effect
///
/// This build does **not** enable object_store's `reqwest` feature — all HTTP
/// goes through a JavaScript `fetch` adapter, so TLS, proxies, DNS and
/// connection pooling are whatever the host runtime is configured with. The
/// options a fetch transport can honour are applied by the transport itself:
///
/// | Option | Honoured here |
/// |---|---|
/// | `allow_http` | yes — plain `http://` requests are rejected unless enabled |
/// | `user_agent` | yes — sent as a `User-Agent` header |
/// | `default_headers` | yes — merged into every request, without overriding explicit headers |
/// | `timeout` | yes — caps the whole request |
/// | `default_content_type`, `content_type_for_suffix` | yes — read by stores when uploading |
/// | `allow_invalid_certificates`, `disable_system_certificates`, `proxy_url`, `proxy_ca_certificate`, `proxy_excludes`, `pool_*`, `http2_*`, `connect_timeout`, `read_timeout` | no — `reqwest`-only |
/// | `withRootCertificate` | unavailable — `Certificate` is `reqwest`-gated upstream |
///
/// The ignored keys are still accepted, and round-trip through
/// [`getConfigValue`](ClientOptions::getConfigValue), so configuration written
/// against a `reqwest` build keeps working unchanged.
///
/// ```js
/// const options = new ClientOptions();
/// options.withAllowHttp(true);
/// options.withUserAgent("my-app/1.0");
/// options.withTimeout(30);
/// options.withDefaultHeaders({ "x-tenant": "acme" });
///
/// const client = new HttpClient(fetchAdapter, options);
/// ```
#[napi]
pub struct ClientOptions {
  /// `ClientOptions` is a consuming builder in Rust, so the NAPI class mutates a
  /// clone and swaps it in. This keeps the JS object mutable and `Send + Sync`.
  inner: Mutex<RsClientOptions>,
}

#[napi]
impl ClientOptions {
  /// Create options with object_store's defaults: a 30s request timeout, a 5s
  /// connect timeout, HTTP/1 only, and `https` only.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Mutex::new(RsClientOptions::new()),
    }
  }

  /// Apply a change, replacing the stored options with the rebuilt value.
  fn update(&self, f: impl FnOnce(RsClientOptions) -> RsClientOptions) {
    let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    let next = f(guard.clone());
    *guard = next;
  }

  fn snapshot(&self) -> RsClientOptions {
    self.inner.lock().unwrap_or_else(|e| e.into_inner()).clone()
  }

  /// Set an option by its `ClientConfigKey` string, e.g. `"allow_http"`,
  /// `"http1_only"`, `"pool_idle_timeout"`.
  #[napi]
  pub fn with_config(&self, key: String, value: String) -> napi::Result<()> {
    let key = parse_config_key(&key)?;
    self.update(|o| o.with_config(key, value));
    Ok(())
  }

  /// Read an option back by its `ClientConfigKey` string, or `null` when unset.
  #[napi]
  pub fn get_config_value(&self, key: String) -> napi::Result<Option<String>> {
    Ok(self.snapshot().get_config_value(&parse_config_key(&key)?))
  }

  /// Permit plain `http://` requests. Only `https` is allowed by default.
  #[napi]
  pub fn with_allow_http(&self, allow_http: bool) {
    self.update(|o| o.with_allow_http(allow_http).with_config(ClientConfigKey::AllowHttp, allow_http.to_string()));
  }

  /// Set the `User-Agent` sent with every request.
  #[napi]
  pub fn with_user_agent(&self, agent: String) -> napi::Result<()> {
    let value = HeaderValue::from_str(&agent)
      .map_err(|e| napi::Error::from_reason(format!("invalid User-Agent {agent:?}: {e}")))?;
    self.update(|o| o.with_user_agent(value).with_config(ClientConfigKey::UserAgent, agent));
    Ok(())
  }

  /// Set headers merged into every request. Per-request headers take precedence.
  #[napi]
  pub fn with_default_headers(&self, headers: HashMap<String, String>) -> napi::Result<()> {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
      map.append(
        HeaderName::from_bytes(name.as_bytes())
          .map_err(|e| napi::Error::from_reason(format!("invalid header name {name:?}: {e}")))?,
        HeaderValue::from_str(&value).map_err(|e| {
          napi::Error::from_reason(format!("invalid value for header {name:?}: {e}"))
        })?,
      );
    }
    self.update(|o| o.with_default_headers(map));
    Ok(())
  }

  /// Set the fallback `Content-Type` used for uploads.
  #[napi]
  pub fn with_default_content_type(&self, mime: String) {
    self.update(|o| o.with_default_content_type(mime));
  }

  /// Set the `Content-Type` used for uploads of files with the given extension.
  #[napi]
  pub fn with_content_type_for_suffix(&self, extension: String, mime: String) {
    self.update(|o| o.with_content_type_for_suffix(extension, mime));
  }

  /// Cap the whole request — from connecting through reading the last body byte.
  ///
  /// The default is 30 seconds.
  #[napi]
  pub fn with_timeout(&self, seconds: f64) -> napi::Result<()> {
    let duration = non_negative_duration(seconds, "timeout")?;
    let formatted = format!("{seconds}s");
    self.update(|o| o.with_timeout(duration).with_config(ClientConfigKey::Timeout, formatted));
    Ok(())
  }

  /// Remove the request timeout, letting requests run indefinitely.
  #[napi]
  pub fn with_timeout_disabled(&self) {
    self.update(|o| o.with_timeout_disabled());
  }

  /// The headers set via [`withDefaultHeaders`](ClientOptions::with_default_headers).
  #[napi]
  pub fn get_default_headers(&self) -> Option<HashMap<String, String>> {
    self
      .snapshot()
      .get_default_headers()
      .map(|map| {
        map.iter()
          .map(|(k, v)| {
            (
              k.as_str().to_string(),
              v.to_str().unwrap_or_default().to_string(),
            )
          })
          .collect()
      })
  }

  /// Resolve the upload `Content-Type` for `path`, honouring both
  /// `withContentTypeForSuffix` and `withDefaultContentType`.
  #[napi]
  pub fn get_content_type(&self, path: String) -> Option<String> {
    self
      .snapshot()
      .get_content_type(&Path::from(path.as_str()))
      .map(str::to_string)
  }

  /// An independent copy of these options.
  ///
  /// Named `clone` to match `PutPayload.clone`; the `#[allow]` is because the
  /// idiomatic name here is reserved by the `Clone` trait, which a `#[napi]`
  /// class cannot implement.
  #[allow(clippy::should_implement_trait)]
  #[napi]
  pub fn clone(&self) -> ClientOptions {
    ClientOptions {
      inner: Mutex::new(self.snapshot()),
    }
  }
}

impl Default for ClientOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl From<&ClientOptions> for RsClientOptions {
  fn from(value: &ClientOptions) -> Self {
    value.snapshot()
  }
}

/// Lets a `ClientOptions` instance be passed **by value**, and therefore
/// optionally, to constructors such as [`HttpClient::new`].
///
/// napi-rs only derives `FromNapiRef` for `#[napi]` classes, which rules out
/// `Option<&ClientOptions>`. Unwrapping the instance and cloning the options out
/// of it is cheap (`ClientOptions::clone` is a plain `Clone`) and sidesteps the
/// borrow checker, since a constructor must not hold a native borrow.
impl FromNapiValue for ClientOptions {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let instance = ClassInstance::<ClientOptions>::from_napi_value(env, value)?;
    let inner: &ClientOptions = &instance;
    Ok(inner.clone())
  }
}

fn parse_config_key(key: &str) -> napi::Result<ClientConfigKey> {
  key
    .parse()
    .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))
}

/// Unwrap an optional NAPI `ClientOptions` into the object_store options,
/// falling back to object_store's defaults when omitted.
fn into_client_options(options: Option<ClientOptions>) -> RsClientOptions {
  options.map(|o| o.snapshot()).unwrap_or_default()
}

fn non_negative_duration(seconds: f64, what: &str) -> napi::Result<Duration> {
  if !seconds.is_finite() || seconds < 0.0 {
    return Err(napi::Error::from_reason(format!(
      "{what} must be a finite, non-negative number of seconds, got {seconds}"
    )));
  }
  Ok(Duration::from_secs_f64(seconds))
}

/// A single HTTP request, as handed to [`HttpClient::execute`].
///
/// `body` is `null` for bodiless requests (GET/HEAD/DELETE) and a `Buffer` for
/// requests that carry a payload.
#[napi(object)]
pub struct HttpRequestOptions {
  /// Absolute request URI, e.g. `"https://example.com/objects/1"`.
  pub url: String,
  /// HTTP method, e.g. `"GET"`, `"PUT"`, `"PROPFIND"`.
  pub method: String,
  /// Request headers; defaults from [`ClientOptions`] fill any gaps.
  pub headers: Option<HashMap<String, String>>,
  /// Request payload, if any.
  pub body: Option<Buffer>,
}

/// A fully buffered HTTP response.
///
/// The body is collected before this is handed back, matching the fetch adapter
/// contract, so `body` is always present (possibly empty).
#[napi(object)]
pub struct HttpResponseResult {
  /// HTTP status code.
  pub status: u16,
  /// Response headers. Repeated headers are comma-joined.
  pub headers: HashMap<String, String>,
  /// Complete response body.
  pub body: Buffer,
}

/// NAPI binding for `object_store::client::HttpClient`.
///
/// `HttpClient` is the transport object_store drives every remote store
/// through. Exposing it directly is useful for talking to an HTTP/WebDAV
/// endpoint that has no object_store equivalent — issuing a `PROPFIND`, a
/// `COPY`, or a request against a bespoke API.
///
/// All I/O is delegated to a **JavaScript fetch adapter**, a
/// `(request) => Promise<response>` function supplied at construction. It
/// receives a `FetchRequest` `{ url, method, headers, body }` and must resolve
/// with a `FetchResponse` `{ status, headers, body }` after reading the full
/// response body:
///
/// ```js
/// import { HttpClient } from "./index.js";
///
/// const fetchAdapter = async (request) => {
///   const res = await fetch(request.url, {
///     method: request.method,
///     headers: request.headers,
///     body: request.body ?? undefined,
///   });
///   return {
///     status: res.status,
///     headers: Object.fromEntries(res.headers),
///     body: Buffer.from(await res.arrayBuffer()),
///   };
/// };
///
/// const client = new HttpClient(fetchAdapter);
/// const res = await client.execute({
///   url: "https://example.com/dav/",
///   method: "PROPFIND",
///   headers: { depth: "1" },
/// });
/// console.log(res.status, res.body.toString("utf8"));
/// ```
///
/// The client holds no base URL, so pass absolute URIs. A non-2xx status is
/// returned as a normal [`HttpResponseResult`] rather than thrown.
#[napi]
pub struct HttpClient {
  inner: RsHttpClient,
}

#[napi]
impl HttpClient {
  /// Create a client whose requests run on napi's own tokio runtime.
  #[napi(constructor)]
  pub fn new(fetch: Unknown, options: Option<ClientOptions>) -> napi::Result<Self> {
    let options = into_client_options(options);
    let connector = fetch_connector(fetch)?;
    let inner = HttpConnector::connect(&connector, &options)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a client whose requests are scheduled on a dedicated [`IoRuntime`].
  ///
  /// Use this to keep object_store's HTTP work off the JavaScript event loop.
  #[napi(factory)]
  pub fn with_runtime(
    fetch: Unknown,
    options: Option<ClientOptions>,
    io_runtime: &IoRuntime,
  ) -> napi::Result<Self> {
    let options = into_client_options(options);
    let connector = io_runtime.connector(fetch)?;
    let inner = HttpConnector::connect(&connector, &options)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Perform `request` and buffer the whole response body.
  ///
  /// Rejects when the request cannot be completed at all (connection failure,
  /// adapter rejection, or timeout). A response that arrives with a non-2xx
  /// status is *not* an error — inspect [`HttpResponseResult::status`].
  #[napi]
  pub async fn execute(&self, request: HttpRequestOptions) -> napi::Result<HttpResponseResult> {
    let response = self
      .inner
      .execute(build_request(request)?)
      .await
      .map_err(http_error)?;

    let (parts, body) = response.into_parts();
    let mut headers: HashMap<String, String> = HashMap::new();
    for (name, value) in parts.headers.iter() {
      headers
        .entry(name.as_str().to_string())
        .and_modify(|existing: &mut String| {
          existing.push_str(", ");
          existing.push_str(value.to_str().unwrap_or_default());
        })
        .or_insert_with(|| value.to_str().unwrap_or_default().to_string());
    }

    let body = body.bytes().await.map_err(http_error)?;

    Ok(HttpResponseResult {
      status: parts.status.as_u16(),
      headers,
      body: Buffer::from(body.to_vec()),
    })
  }
}

fn build_request(options: HttpRequestOptions) -> napi::Result<RsHttpRequest> {
  let method = Method::from_bytes(options.method.as_bytes())
    .map_err(|e| napi::Error::from_reason(format!("invalid HTTP method {:?}: {e}", options.method)))?;
  let uri: Uri = options
    .url
    .parse()
    .map_err(|e| napi::Error::from_reason(format!("invalid request URL {:?}: {e}", options.url)))?;

  let mut headers = HeaderMap::new();
  for (name, value) in options.headers.unwrap_or_default() {
    headers.append(
      HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| napi::Error::from_reason(format!("invalid header name {name:?}: {e}")))?,
      HeaderValue::from_str(&value).map_err(|e| {
        napi::Error::from_reason(format!("invalid value for header {name:?}: {e}"))
      })?,
    );
  }

  let body = match options.body {
    Some(body) if !body.is_empty() => HttpRequestBody::from(body.to_vec()),
    _ => HttpRequestBody::empty(),
  };

  let mut request = RsHttpRequest::new(body);
  *request.method_mut() = method;
  *request.uri_mut() = uri;
  *request.headers_mut() = headers;
  Ok(request)
}

/// Render an `HttpError` for JavaScript, keeping the kind so callers can tell a
/// timeout apart from a connection failure.
fn http_error(error: RsHttpError) -> napi::Error {
  napi::Error::from_reason(format!(
    "HTTP request failed ({:?}): {}",
    error.kind(),
    error
  ))
}
