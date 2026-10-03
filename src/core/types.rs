use std::sync::Arc;
use napi_derive::napi;
use object_store::{ObjectMeta as ObjObjectMeta, ObjectStore as DynObjectStore};

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
pub struct RangeInput {
  pub start: i64,
  pub end: i64,
}

#[derive(Clone)]
#[napi]
pub struct ObjectStore {
  pub(crate) inner: Arc<dyn DynObjectStore>,
}

#[napi]
pub struct ParseUrlResult {
  pub(crate) store: ObjectStore,
  pub(crate) path: String,
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
