use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::{
  parse_url_opts, GetOptions as ObjGetOptions, GetRange, ObjectMeta as ObjObjectMeta,
  ObjectStore as DynObjectStore, ObjectStoreExt, PutMode, PutOptions as ObjPutOptions, PutPayload,
  UpdateVersion,
};
use url::Url;

#[napi(object)]
pub struct ObjectMeta {
  pub location: String,
  pub last_modified: String,
  pub size: i64,
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

impl From<ObjObjectMeta> for ObjectMeta {
  fn from(meta: ObjObjectMeta) -> Self {
    ObjectMeta {
      location: meta.location.to_string(),
      last_modified: meta.last_modified.to_rfc3339(),
      size: meta.size as i64,
      e_tag: meta.e_tag,
      version: meta.version,
    }
  }
}

#[napi(object)]
pub struct PutResult {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct GetOptions {
  pub range_start: Option<i64>,
  pub range_end: Option<i64>,
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
}

#[napi(object)]
pub struct PutOptions {
  pub mode: Option<String>,
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct RangeInput {
  pub start: i64,
  pub end: i64,
}

#[derive(Clone)]
#[napi]
pub struct ObjectStore {
  inner: Arc<dyn DynObjectStore>,
}

#[napi]
pub struct ParseUrlResult {
  store: ObjectStore,
  path: String,
}

#[napi]
impl ParseUrlResult {
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
  #[napi(constructor)]
  pub fn new(url: Option<String>, options: Option<HashMap<String, String>>) -> Result<Self> {
    if let Some(u) = url {
      if u.trim().is_empty() || u == "memory://" || u == "memory:///" {
        Ok(Self {
          inner: Arc::new(InMemory::new()),
        })
      } else {
        let parsed_url = Url::parse(&u).map_err(|e| Error::from_reason(e.to_string()))?;
        let opts = options.unwrap_or_default();
        let (store, _path) = parse_url_opts(&parsed_url, opts)
          .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self {
          inner: Arc::from(store),
        })
      }
    } else {
      Ok(Self {
        inner: Arc::new(InMemory::new()),
      })
    }
  }

  #[napi(factory)]
  pub fn memory() -> Self {
    Self {
      inner: Arc::new(InMemory::new()),
    }
  }

  #[napi(factory)]
  pub fn local(root_path: String) -> Result<Self> {
    let store = LocalFileSystem::new_with_prefix(&root_path)
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(store),
    })
  }

  #[napi]
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

  #[napi]
  pub async fn put(&self, path: String, payload: Uint8Array) -> Result<PutResult> {
    let location = Path::from(path.as_str());
    let put_payload = PutPayload::from(payload.to_vec());
    let res = self
      .inner
      .put(&location, put_payload)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    payload: Uint8Array,
    options: Option<PutOptions>,
  ) -> Result<PutResult> {
    let location = Path::from(path.as_str());
    let put_payload = PutPayload::from(payload.to_vec());

    let put_mode = if let Some(opts) = &options {
      match opts.mode.as_deref() {
        Some("create") => PutMode::Create,
        Some("update") => PutMode::Update(UpdateVersion {
          e_tag: opts.e_tag.clone(),
          version: opts.version.clone(),
        }),
        _ => PutMode::Overwrite,
      }
    } else {
      PutMode::Overwrite
    };

    let put_options = ObjPutOptions::from(put_mode);
    let res = self
      .inner
      .put_opts(&location, put_payload, put_options)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  #[napi]
  pub async fn get(&self, path: String) -> Result<Buffer> {
    let location = Path::from(path.as_str());
    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    let bytes = res
      .bytes()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }

  #[napi]
  pub async fn get_opts(&self, path: String, options: Option<GetOptions>) -> Result<Buffer> {
    let location = Path::from(path.as_str());
    let mut get_opts = ObjGetOptions::default();

    if let Some(opts) = options {
      if let (Some(start), Some(end)) = (opts.range_start, opts.range_end) {
        if start >= 0 && end >= start {
          get_opts.range = Some(GetRange::Bounded(Range {
            start: start as u64,
            end: end as u64,
          }));
        }
      } else if let Some(start) = opts.range_start {
        if start >= 0 {
          get_opts.range = Some(GetRange::Offset(start as u64));
        }
      } else if let Some(suffix) = opts.range_end {
        if suffix >= 0 {
          get_opts.range = Some(GetRange::Suffix(suffix as u64));
        }
      }

      if let Some(if_match) = opts.if_match {
        get_opts.if_match = Some(if_match);
      }
      if let Some(if_none_match) = opts.if_none_match {
        get_opts.if_none_match = Some(if_none_match);
      }
    }

    let res = self
      .inner
      .get_opts(&location, get_opts)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    let bytes = res
      .bytes()
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<RangeInput>) -> Result<Vec<Buffer>> {
    let location = Path::from(path.as_str());
    let rust_ranges: Vec<Range<u64>> = ranges
      .into_iter()
      .map(|r| Range {
        start: r.start.max(0) as u64,
        end: r.end.max(0) as u64,
      })
      .collect();

    let bytes_vec = self
      .inner
      .get_ranges(&location, &rust_ranges)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(
      bytes_vec
        .into_iter()
        .map(|b| Buffer::from(b.as_ref()))
        .collect(),
    )
  }

  #[napi]
  pub async fn head(&self, path: String) -> Result<ObjectMeta> {
    let location = Path::from(path.as_str());
    let meta = self
      .inner
      .head(&location)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(ObjectMeta::from(meta))
  }

  #[napi]
  pub async fn delete(&self, path: String) -> Result<()> {
    let location = Path::from(path.as_str());
    self
      .inner
      .delete(&location)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn list(&self, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let mut stream = self.inner.list(prefix_path.as_ref());

    let mut result = Vec::new();
    while let Some(meta_res) = stream.next().await {
      let meta = meta_res.map_err(|e| Error::from_reason(e.to_string()))?;
      result.push(ObjectMeta::from(meta));
    }

    Ok(result)
  }

  #[napi]
  pub async fn copy(&self, from: String, to: String) -> Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .copy(&from_path, &to_path)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    self
      .inner
      .rename(&from_path, &to_path)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
  }
}

#[napi]
pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParseUrlResult> {
  ObjectStore::parse_url(url, options)
}
