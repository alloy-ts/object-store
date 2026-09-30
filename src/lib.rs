use bytes::Bytes;
use futures::StreamExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::{
    local::LocalFileSystem,
    memory::InMemory,
    parse_url_opts,
    path::Path,
    ObjectMeta as RSObjectMeta,
    ObjectStore as ObjectStoreTrait,
    ObjectStoreExt,
    PutPayload,
};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

#[napi(object)]
pub struct ObjectMeta {
  pub location: String,
  pub last_modified: i64,
  pub size: f64,
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

fn convert_meta(meta: &RSObjectMeta) -> ObjectMeta {
  ObjectMeta {
    location: meta.location.to_string(),
    last_modified: meta.last_modified.timestamp_millis(),
    size: meta.size as f64,
    e_tag: meta.e_tag.clone(),
    version: meta.version.clone(),
  }
}

#[napi(object)]
pub struct PutResult {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct GetResult {
  pub bytes: Buffer,
  pub meta: ObjectMeta,
}

#[napi(object)]
pub struct ListResult {
  pub objects: Vec<ObjectMeta>,
  pub common_prefixes: Vec<String>,
}

#[napi(object)]
pub struct Range {
  pub start: f64,
  pub end: f64,
}

#[napi]
pub struct ObjectStore {
  inner: Arc<dyn ObjectStoreTrait>,
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
    let parsed_url = Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let opts = options.unwrap_or_default();
    let (store, _path) = parse_url_opts(&parsed_url, opts)
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: store.into(),
    })
  }

  #[napi]
  pub async fn put(&self, path: String, data: Buffer) -> napi::Result<PutResult> {
    let location = Path::from(path.as_str());
    let payload = PutPayload::from(Bytes::from(data.to_vec()));
    let res = self
      .inner
      .put(&location, payload)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  #[napi]
  pub async fn get(&self, path: String) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
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

  #[napi]
  pub async fn get_with_meta(&self, path: String) -> napi::Result<GetResult> {
    let location = Path::from(path.as_str());
    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    let meta = convert_meta(&res.meta);
    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(GetResult {
      bytes: Buffer::from(bytes.as_ref()),
      meta,
    })
  }

  #[napi]
  pub async fn head(&self, path: String) -> napi::Result<ObjectMeta> {
    let location = Path::from(path.as_str());
    let meta = self
      .inner
      .head(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(convert_meta(&meta))
  }

  #[napi]
  pub async fn delete(&self, path: String) -> napi::Result<()> {
    let location = Path::from(path.as_str());
    self
      .inner
      .delete(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn list(&self, prefix: Option<String>) -> napi::Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let mut stream = self.inner.list(prefix_path.as_ref());
    let mut results = Vec::new();
    while let Some(item) = stream.next().await {
      let meta = item.map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      results.push(convert_meta(&meta));
    }
    Ok(results)
  }

  #[napi]
  pub async fn list_with_delimiter(&self, prefix: Option<String>) -> napi::Result<ListResult> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let res = self
      .inner
      .list_with_delimiter(prefix_path.as_ref())
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(ListResult {
      objects: res.objects.iter().map(convert_meta).collect(),
      common_prefixes: res.common_prefixes.iter().map(|p| p.to_string()).collect(),
    })
  }

  #[napi]
  pub async fn copy(&self, from: String, to: String) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .copy(&from_path, &to_path)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .rename(&from_path, &to_path)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<Range>) -> napi::Result<Vec<Buffer>> {
    let location = Path::from(path.as_str());
    let range_ops: Vec<std::ops::Range<u64>> = ranges
      .iter()
      .map(|r| (r.start as u64)..(r.end as u64))
      .collect();
    let results = self
      .inner
      .get_ranges(&location, &range_ops)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(
      results
        .into_iter()
        .map(|b| Buffer::from(b.as_ref()))
        .collect(),
    )
  }
}
