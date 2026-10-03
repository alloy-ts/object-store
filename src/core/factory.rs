use std::collections::HashMap;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use url::Url;

use crate::core::types::{ObjectStore, ParsedUrl};

pub fn store_in_memory() -> ObjectStore {
  ObjectStore {
    inner: Arc::new(object_store::memory::InMemory::new()),
  }
}

pub fn store_local(path: String) -> Result<ObjectStore> {
  let local = object_store::local::LocalFileSystem::new_with_prefix(path)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ObjectStore {
    inner: Arc::new(local),
  })
}

pub fn store_from_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ObjectStore> {
  if url == "memory" || url.starts_with("memory://") {
    return Ok(store_in_memory());
  }
  let parsed = Url::parse(&url).map_err(|e| Error::from_reason(e.to_string()))?;
  let opts = options.unwrap_or_default();
  let (store, _path) = object_store::parse_url_opts(&parsed, opts)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ObjectStore {
    inner: store.into(),
  })
}

#[napi]
pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParsedUrl> {
  if url == "memory" || url.starts_with("memory://") {
    let path = url.strip_prefix("memory://").unwrap_or("").to_string();
    return Ok(ParsedUrl {
      store: store_in_memory(),
      path,
    });
  }
  let parsed = Url::parse(&url).map_err(|e| Error::from_reason(e.to_string()))?;
  let opts = options.unwrap_or_default();
  let (store, path) = object_store::parse_url_opts(&parsed, opts)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ParsedUrl {
    store: ObjectStore {
      inner: store.into(),
    },
    path: path.to_string(),
  })
}
