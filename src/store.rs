use napi_derive::napi;
use object_store::{local::LocalFileSystem, memory::InMemory, ObjectStore as ObjectStoreTrait};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

#[napi]
pub struct ObjectStore {
  pub(crate) inner: Arc<dyn ObjectStoreTrait>,
}

#[napi]
impl ObjectStore {
  #[napi(factory)]
  pub fn create_in_memory() -> Self {
    Self {
      inner: Arc::new(InMemory::new()),
    }
  }

  #[napi(factory)]
  pub fn create_local(root_path: String) -> napi::Result<Self> {
    let local = LocalFileSystem::new_with_prefix(root_path)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(local),
    })
  }

  #[napi(factory)]
  pub fn parse_url(url: String, options: Option<HashMap<String, String>>) -> napi::Result<Self> {
    let _ = options; // local/memory stores take no URL options
    let parsed_url =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let store: Arc<dyn ObjectStoreTrait> = match parsed_url.scheme() {
      "file" => Arc::new(
        LocalFileSystem::new_with_prefix(parsed_url.path())
          .map_err(|e| napi::Error::from_reason(e.to_string()))?,
      ),
      "memory" => Arc::new(InMemory::new()),
      scheme => {
        return Err(napi::Error::from_reason(format!(
          "unsupported scheme '{scheme}': only file:// and memory:// are supported \
           (cloud/reqwest features are disabled; http-base requires a custom HttpConnector \
           to use http(s):// URLs)"
        )))
      }
    };
    Ok(Self { inner: store })
  }
}
