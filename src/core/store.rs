use napi_derive::napi;
use object_store::{
    local::LocalFileSystem, memory::InMemory, parse_url_opts, ObjectStore as ObjectStoreTrait,
};
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
    let parsed_url =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let opts = options.unwrap_or_default();
    let (store, _path) = parse_url_opts(&parsed_url, opts)
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: store.into(),
    })
  }
}
