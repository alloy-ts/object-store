use std::collections::HashMap;
use std::sync::Arc;
use napi::bindgen_prelude::*;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::parse_url_opts;
use url::Url;

use super::types::{ObjectStore, ParseUrlResult};

pub fn new(url: Option<String>, options: Option<HashMap<String, String>>) -> Result<ObjectStore> {
  if let Some(u) = url {
    if u.trim().is_empty() || u == "memory://" || u == "memory:///" {
      Ok(ObjectStore {
        inner: Arc::new(InMemory::new()),
      })
    } else {
      let parsed_url = Url::parse(&u).map_err(|e| Error::from_reason(e.to_string()))?;
      let opts = options.unwrap_or_default();
      let (store, _path) = parse_url_opts(&parsed_url, opts)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(ObjectStore {
        inner: Arc::from(store),
      })
    }
  } else {
    Ok(ObjectStore {
      inner: Arc::new(InMemory::new()),
    })
  }
}

pub fn memory() -> ObjectStore {
  ObjectStore {
    inner: Arc::new(InMemory::new()),
  }
}

pub fn local(root_path: String) -> Result<ObjectStore> {
  let store = LocalFileSystem::new_with_prefix(&root_path)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ObjectStore {
    inner: Arc::new(store),
  })
}

pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParseUrlResult> {
  let parsed_url = Url::parse(&url).map_err(|e| Error::from_reason(e.to_string()))?;
  let opts = options.unwrap_or_default();
  let (store, path) =
    parse_url_opts(&parsed_url, opts).map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(ParseUrlResult {
    store: ObjectStore {
      inner: Arc::from(store),
    },
    path: path.to_string(),
  })
}
