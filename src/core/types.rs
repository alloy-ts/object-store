use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::ObjectMeta as RSObjectMeta;

#[napi(object)]
pub struct ObjectMeta {
  pub location: String,
  pub last_modified: i64,
  pub size: f64,
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

pub fn convert_meta(meta: &RSObjectMeta) -> ObjectMeta {
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

#[napi(object)]
pub struct GetOptionsInput {
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
  pub range_start: Option<f64>,
  pub range_end: Option<f64>,
}

#[napi(object)]
pub struct UpdateVersionInput {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct PutOptionsInput {
  pub mode_overwrite: Option<bool>,
  pub mode_create: Option<bool>,
  pub mode_update: Option<UpdateVersionInput>,
}
