use std::collections::HashMap;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::ObjectStore as ObjectStoreTrait;

use crate::core::copy::{store_copy, store_copy_opts};
use crate::core::delete::store_delete;
use crate::core::factory::{store_from_url, store_in_memory, store_local};
use crate::core::get::{store_get, store_get_opts, store_get_range, store_get_ranges};
use crate::core::head::store_head;
use crate::core::list::{store_list, store_list_with_delimiter};
use crate::core::put::{store_put, store_put_opts};
use crate::core::rename::{store_rename, store_rename_opts};

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

#[napi(object)]
pub struct GetOptionsParam {
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
  pub if_modified_since: Option<String>,
  pub if_unmodified_since: Option<String>,
  pub range: Option<RangeParam>,
  pub version: Option<String>,
  pub head: Option<bool>,
}

#[napi(object)]
pub struct UpdateVersion {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct PutOptionsParam {
  pub mode: Option<String>,
  pub version: Option<UpdateVersion>,
}

#[napi(object)]
pub struct CopyOptionsParam {
  pub mode: Option<String>,
}

#[napi(object)]
pub struct RenameOptionsParam {
  pub mode: Option<String>,
}

pub fn convert_meta(meta: &object_store::ObjectMeta) -> ObjectMeta {
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
  pub(crate) inner: Arc<dyn ObjectStoreTrait>,
}

#[napi]
impl ObjectStore {
  #[napi(factory)]
  pub fn in_memory() -> Self {
    store_in_memory()
  }

  #[napi(factory)]
  pub fn local(path: String) -> Result<Self> {
    store_local(path)
  }

  #[napi(factory)]
  pub fn from_url(url: String, options: Option<HashMap<String, String>>) -> Result<Self> {
    store_from_url(url, options)
  }

  #[napi]
  pub async fn put(&self, path: String, bytes: Buffer) -> Result<PutResult> {
    store_put(self, path, bytes).await
  }

  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    bytes: Buffer,
    options: PutOptionsParam,
  ) -> Result<PutResult> {
    store_put_opts(self, path, bytes, options).await
  }

  #[napi]
  pub async fn get(&self, path: String) -> Result<GetResult> {
    store_get(self, path).await
  }

  #[napi]
  pub async fn get_opts(&self, path: String, options: GetOptionsParam) -> Result<GetResult> {
    store_get_opts(self, path, options).await
  }

  #[napi]
  pub async fn get_range(&self, path: String, start: i64, length: i64) -> Result<Buffer> {
    store_get_range(self, path, start, length).await
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<RangeParam>) -> Result<Vec<Buffer>> {
    store_get_ranges(self, path, ranges).await
  }

  #[napi]
  pub async fn head(&self, path: String) -> Result<ObjectMeta> {
    store_head(self, path).await
  }

  #[napi]
  pub async fn delete(&self, path: String) -> Result<()> {
    store_delete(self, path).await
  }

  #[napi]
  pub async fn list(&self, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
    store_list(self, prefix).await
  }

  #[napi]
  pub async fn list_with_delimiter(&self, prefix: Option<String>) -> Result<ListResult> {
    store_list_with_delimiter(self, prefix).await
  }

  #[napi]
  pub async fn copy(&self, from: String, to: String) -> Result<()> {
    store_copy(self, from, to).await
  }

  #[napi]
  pub async fn copy_opts(
    &self,
    from: String,
    to: String,
    options: CopyOptionsParam,
  ) -> Result<()> {
    store_copy_opts(self, from, to, options).await
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> Result<()> {
    store_rename(self, from, to).await
  }

  #[napi]
  pub async fn rename_opts(
    &self,
    from: String,
    to: String,
    options: RenameOptionsParam,
  ) -> Result<()> {
    store_rename_opts(self, from, to, options).await
  }
}

#[napi]
pub struct ParsedUrl {
  pub(crate) store: ObjectStore,
  pub(crate) path: String,
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
