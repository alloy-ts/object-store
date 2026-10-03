use std::collections::HashMap;
use std::sync::Arc;

use futures::TryStreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{ObjectStore as ObjectStoreTrait, ObjectStoreExt};
use url::Url;

#[napi(object)]
pub struct ObjectMeta {
  pub location: String,
  pub last_modified: String,
  pub size: i64,
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct GetResult {
  pub bytes: Buffer,
  pub meta: ObjectMeta,
}

#[napi(object)]
pub struct PutResult {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct ListResult {
  pub objects: Vec<ObjectMeta>,
  pub common_prefixes: Vec<String>,
}

#[napi(object)]
pub struct RangeParam {
  pub start: i64,
  pub length: i64,
}

fn convert_meta(meta: &object_store::ObjectMeta) -> ObjectMeta {
  ObjectMeta {
    location: meta.location.to_string(),
    last_modified: meta.last_modified.to_rfc3339(),
    size: meta.size as i64,
    e_tag: meta.e_tag.clone(),
    version: meta.version.clone(),
  }
}

#[napi]
#[derive(Clone)]
pub struct ObjectStore {
  inner: Arc<dyn ObjectStoreTrait>,
}

#[napi]
pub struct ParsedUrl {
  store: ObjectStore,
  path: String,
}

#[napi]
impl ParsedUrl {
  #[napi(getter)]
  pub fn store(&self) -> ObjectStore {
    self.store.clone()
  }

  #[napi(getter)]
  pub fn path(&self) -> String {
    self.path.clone()
  }
}

#[napi]
impl ObjectStore {
  #[napi(factory)]
  pub fn in_memory() -> Self {
    Self {
      inner: Arc::new(object_store::memory::InMemory::new()),
    }
  }

  #[napi(factory)]
  pub fn local(path: String) -> Result<Self> {
    let local = object_store::local::LocalFileSystem::new_with_prefix(path)
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(local),
    })
  }

  #[napi(factory)]
  pub fn from_url(
    url: String,
    options: Option<HashMap<String, String>>,
  ) -> Result<Self> {
    if url == "memory" || url.starts_with("memory://") {
      return Ok(Self::in_memory());
    }
    let parsed = Url::parse(&url).map_err(|e| Error::from_reason(e.to_string()))?;
    let opts = options.unwrap_or_default();
    let (store, _path) = object_store::parse_url_opts(&parsed, opts)
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: store.into(),
    })
  }

  #[napi]
  pub async fn put(&self, path: String, bytes: Buffer) -> Result<PutResult> {
    let p = Path::from(path.as_str());
    let payload = object_store::PutPayload::from(bytes.to_vec());
    let res = self
      .inner
      .put(&p, payload)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  #[napi]
  pub async fn get(&self, path: String) -> Result<GetResult> {
    let p = Path::from(path.as_str());
    let res = self
      .inner
      .get(&p)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    let meta = convert_meta(&res.meta);
    let bytes = res
      .bytes()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(GetResult {
      bytes: Buffer::from(bytes.to_vec()),
      meta,
    })
  }

  #[napi]
  pub async fn get_range(&self, path: String, start: i64, length: i64) -> Result<Buffer> {
    let p = Path::from(path.as_str());
    let range = (start as u64)..(start as u64 + length as u64);
    let bytes = self
      .inner
      .get_range(&p, range)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.to_vec()))
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<RangeParam>) -> Result<Vec<Buffer>> {
    let p = Path::from(path.as_str());
    let rs: Vec<std::ops::Range<u64>> = ranges
      .iter()
      .map(|r| (r.start as u64)..(r.start as u64 + r.length as u64))
      .collect();
    let results = self
      .inner
      .get_ranges(&p, &rs)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(results.into_iter().map(|b| Buffer::from(b.to_vec())).collect())
  }

  #[napi]
  pub async fn head(&self, path: String) -> Result<ObjectMeta> {
    let p = Path::from(path.as_str());
    let meta = self
      .inner
      .head(&p)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(convert_meta(&meta))
  }

  #[napi]
  pub async fn delete(&self, path: String) -> Result<()> {
    let p = Path::from(path.as_str());
    self
      .inner
      .delete(&p)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn list(&self, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
    let p = prefix.map(|s| Path::from(s.as_str()));
    let stream = self.inner.list(p.as_ref());
    let metas: Vec<object_store::ObjectMeta> = stream
      .try_collect()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(metas.iter().map(convert_meta).collect())
  }

  #[napi]
  pub async fn list_with_delimiter(&self, prefix: Option<String>) -> Result<ListResult> {
    let p = prefix.map(|s| Path::from(s.as_str()));
    let res = self
      .inner
      .list_with_delimiter(p.as_ref())
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    let objects = res.objects.iter().map(convert_meta).collect();
    let common_prefixes = res
      .common_prefixes
      .iter()
      .map(|cp| cp.to_string())
      .collect();
    Ok(ListResult {
      objects,
      common_prefixes,
    })
  }

  #[napi]
  pub async fn copy(&self, from: String, to: String) -> Result<()> {
    let from_p = Path::from(from.as_str());
    let to_p = Path::from(to.as_str());
    self
      .inner
      .copy(&from_p, &to_p)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> Result<()> {
    let from_p = Path::from(from.as_str());
    let to_p = Path::from(to.as_str());
    self
      .inner
      .rename(&from_p, &to_p)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }
}

#[napi]
pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParsedUrl> {
  if url == "memory" || url.starts_with("memory://") {
    let path = url.strip_prefix("memory://").unwrap_or("").to_string();
    return Ok(ParsedUrl {
      store: ObjectStore::in_memory(),
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
