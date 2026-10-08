//! Run object_store HTTP work on a dedicated tokio runtime.
//!
//! object_store ships [`SpawnService`], a generic [`HttpService`] adapter that
//! moves every request onto a caller-supplied [`tokio::runtime::Handle`] and
//! streams the response back across the runtime boundary. It is **not** gated on
//! the `reqwest` feature — only its documented example, `SpawnedReqwestConnector`,
//! is — so it is reused verbatim here and driven by our JavaScript-fetch
//! transport rather than by `reqwest`.
//!
//! ```js
//! import { HttpClient, IoRuntime } from "./index.js";
//!
//! // 4 worker threads dedicated to HTTP, independent of the JS event loop.
//! const io = new IoRuntime(4);
//! const client = HttpClient.withRuntime(fetchAdapter, undefined, io);
//! ```

use std::sync::Arc;

use napi::bindgen_prelude::Unknown;
use napi_derive::napi;
use object_store::client::{HttpClient as RsHttpClient, HttpConnector};
use object_store::ClientOptions;
use tokio::runtime::{Handle, Runtime};

use crate::core::{fetch_connector, NodeFetchConnector};

pub use object_store::client::SpawnService;

/// An [`HttpConnector`] that performs all request I/O on a dedicated tokio
/// runtime instead of the runtime driving the JavaScript event loop.
///
/// Useful to keep CPU-bound work on the main loop from starving in-flight
/// requests (and vice versa). The response body is still produced by the
/// JavaScript fetch adapter; only the task scheduling is relocated.
pub(crate) struct SpawnedNodeFetchConnector {
  connector: NodeFetchConnector,
  runtime: Handle,
}

impl SpawnedNodeFetchConnector {
  pub(crate) fn new(fetch: Unknown, runtime: Handle) -> napi::Result<Self> {
    Ok(Self {
      connector: fetch_connector(fetch)?,
      runtime,
    })
  }
}

impl std::fmt::Debug for SpawnedNodeFetchConnector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SpawnedNodeFetchConnector").finish()
  }
}

impl HttpConnector for SpawnedNodeFetchConnector {
  fn connect(&self, options: &ClientOptions) -> object_store::Result<RsHttpClient> {
    let service = self.connector.service(options);
    Ok(RsHttpClient::new(SpawnService::new(
      service,
      self.runtime.clone(),
    )))
  }
}

/// A tokio runtime reserved for object_store HTTP I/O.
///
/// The runtime is multi-threaded and independent of the one napi uses for the
/// JavaScript event loop, so blocking or CPU-heavy work in JS callbacks will
/// not stall requests in flight. Keep a reference to the `IoRuntime` for as long
/// as the clients built from it are in use — dropping it shuts the runtime down
/// and aborts their requests.
///
/// ```js
/// const io = new IoRuntime(4);          // 4 worker threads
/// const one = new IoRuntime(1);         // single worker thread
/// const dflt = new IoRuntime();         // tokio's default worker count
/// ```
#[napi]
pub struct IoRuntime {
  /// Dropping the runtime shuts it down, so the owner is kept alongside the
  /// handle rather than letting the handle keep it alive on its own.
  _runtime: Arc<Runtime>,
  handle: Handle,
}

#[napi]
impl IoRuntime {
  /// Create a runtime for HTTP I/O.
  ///
  /// `threads` is the number of worker threads; omit it to use tokio's default
  /// (one per available core).
  #[napi(constructor)]
  pub fn new(threads: Option<u32>) -> napi::Result<Self> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Some(threads) = threads {
      if threads == 0 {
        return Err(napi::Error::from_reason(
          "IoRuntime requires at least 1 thread",
        ));
      }
      builder.worker_threads(threads as usize);
    }

    let runtime = builder
      .build()
      .map_err(|e| napi::Error::from_reason(format!("failed to create IoRuntime: {e}")))?;

    Ok(Self {
      handle: runtime.handle().clone(),
      _runtime: Arc::new(runtime),
    })
  }

  pub(crate) fn handle(&self) -> Handle {
    self.handle.clone()
  }

  pub(crate) fn connector(&self, fetch: Unknown) -> napi::Result<SpawnedNodeFetchConnector> {
    SpawnedNodeFetchConnector::new(fetch, self.handle())
  }
}
