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
pub struct GetRangeInput {
  pub start: Option<f64>,
  pub end: Option<f64>,
  pub offset: Option<f64>,
  pub suffix: Option<f64>,
}

#[napi(object)]
pub struct GetOptionsInput {
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
  pub if_modified_since: Option<i64>,
  pub if_unmodified_since: Option<i64>,
  pub range: Option<GetRangeInput>,
  pub version: Option<String>,
  pub head: Option<bool>,
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

#[napi(object)]
pub struct CopyOptionsInput {
  pub if_not_exists: Option<bool>,
}

#[napi(object)]
pub struct RenameOptionsInput {
  pub target_mode_overwrite: Option<bool>,
  pub target_mode_create: Option<bool>,
}

#[napi(object)]
pub struct HeadOptionsInput {
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
  pub if_modified_since: Option<i64>,
  pub if_unmodified_since: Option<i64>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct DeleteOptionsInput {
  pub dummy: Option<bool>,
}

#[napi(object)]
pub struct ListOptionsInput {
  pub offset: Option<String>,
}

#[napi(object)]
pub struct PutMultipartOptionsInput {
  pub dummy: Option<bool>,
}
