use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use chrono::DateTime;
use object_store::ObjectMeta as RSObjectMeta;
use object_store::list::PaginatedListOptions as RSPaginatedListOptions;
use object_store::path::Path;
use object_store::{
  Attribute, AttributeValue, Attributes, GetOptions, GetRange, PutMode, PutMultipartOptions,
  PutOptions, TagSet, UpdateVersion,
};
use std::borrow::Cow;
use std::collections::HashMap;

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

/// Build an `object_store::ObjectMeta` from the JS `ObjectMeta` shape.
///
/// Needed to feed [`object_store::buffered::BufReader`] /
/// [`object_store::buffered::BufWriter`], which require a real `ObjectMeta`.
pub fn build_object_meta(meta: &ObjectMeta) -> RSObjectMeta {
  RSObjectMeta {
    location: Path::from(meta.location.as_str()),
    last_modified: DateTime::from_timestamp_millis(meta.last_modified).unwrap_or_default(),
    size: meta.size as u64,
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
  pub range: Range,
  pub attributes: HashMap<String, String>,
}

#[napi(object)]
pub struct ListResult {
  pub objects: Vec<ObjectMeta>,
  pub common_prefixes: Vec<String>,
}

/// Options for a paginated list request
///
/// Mirrors `object_store::list::PaginatedListOptions`. The Rust-only
/// `extensions` field has no JS counterpart and is left empty.
#[napi(object)]
pub struct PaginatedListOptionsInput {
  /// Path to start listing from. The object at this key is not included.
  pub offset: Option<String>,
  /// A delimiter used to group keys with a common prefix. Some stores only
  /// support `/`.
  pub delimiter: Option<String>,
  /// The maximum number of paths (objects plus common prefixes) to return.
  pub max_keys: Option<u32>,
  /// A page token from a previous request. Behaviour is implementation
  /// defined if the previous request used a different prefix or options.
  pub page_token: Option<String>,
}

/// A [`ListResult`] with an optional pagination token
#[napi(object)]
pub struct PaginatedListResult {
  /// The list result
  pub result: ListResult,
  /// If the result set was truncated, the token to fetch the next results
  pub page_token: Option<String>,
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
  pub tags: Option<HashMap<String, String>>,
  pub attributes: Option<HashMap<String, String>>,
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
pub struct ListOptionsInput {
  pub offset: Option<String>,
}

#[napi(object)]
pub struct PutMultipartOptionsInput {
  pub tags: Option<HashMap<String, String>>,
  pub attributes: Option<HashMap<String, String>>,
}

/// Build an `object_store::GetOptions` from the JS `GetOptionsInput` shape.
/// Shared by every store binding so the parsing logic lives in one place.
pub fn build_get_options(input: &GetOptionsInput) -> GetOptions {
  let mut opts = GetOptions::default();
  if let Some(if_match) = &input.if_match {
    opts.if_match = Some(if_match.clone());
  }
  if let Some(if_none_match) = &input.if_none_match {
    opts.if_none_match = Some(if_none_match.clone());
  }
  if let Some(ms) = input.if_modified_since {
    if let Some(dt) = DateTime::from_timestamp_millis(ms) {
      opts.if_modified_since = Some(dt);
    }
  }
  if let Some(ms) = input.if_unmodified_since {
    if let Some(dt) = DateTime::from_timestamp_millis(ms) {
      opts.if_unmodified_since = Some(dt);
    }
  }
  if let Some(r) = &input.range {
    if let (Some(start), Some(end)) = (r.start, r.end) {
      opts.range = Some(GetRange::Bounded((start as u64)..(end as u64)));
    } else if let Some(offset) = r.offset {
      opts.range = Some(GetRange::Offset(offset as u64));
    } else if let Some(suffix) = r.suffix {
      opts.range = Some(GetRange::Suffix(suffix as u64));
    }
  }
  if let Some(v) = &input.version {
    opts.version = Some(v.clone());
  }
  if let Some(h) = input.head {
    opts.head = h;
  }
  opts
}

/// Build an `object_store::PutOptions` from the JS `PutOptionsInput` shape.
/// Shared by every store binding so the parsing logic lives in one place.
pub fn build_put_options(input: &PutOptionsInput) -> napi::Result<PutOptions> {
  let mode = if input.mode_create == Some(true) {
    PutMode::Create
  } else if let Some(update) = &input.mode_update {
    PutMode::Update(UpdateVersion {
      e_tag: update.e_tag.clone(),
      version: update.version.clone(),
    })
  } else {
    PutMode::Overwrite
  };
  Ok(PutOptions {
    mode,
    tags: build_tag_set(input.tags.as_ref()),
    attributes: build_attributes(input.attributes.as_ref())?,
    extensions: Default::default(),
  })
}

/// Build an `object_store::PutMultipartOptions` from the JS
/// `PutMultipartOptionsInput` shape.
pub fn build_put_multipart_options(
  input: Option<&PutMultipartOptionsInput>,
) -> napi::Result<PutMultipartOptions> {
  Ok(PutMultipartOptions {
    tags: build_tag_set(input.and_then(|i| i.tags.as_ref())),
    attributes: build_attributes(input.and_then(|i| i.attributes.as_ref()))?,
    extensions: Default::default(),
  })
}

/// Build an `object_store::TagSet`, ordering keys so the encoded form is
/// deterministic regardless of JavaScript object iteration order.
pub fn build_tag_set(input: Option<&HashMap<String, String>>) -> TagSet {
  let mut tags = TagSet::default();
  let Some(input) = input else {
    return tags;
  };
  let mut pairs: Vec<(&String, &String)> = input.iter().collect();
  pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
  for (key, value) in pairs {
    tags.push(key, value);
  }
  tags
}

/// Build `object_store::Attributes` from the JS key/value shape.
pub fn build_attributes(input: Option<&HashMap<String, String>>) -> napi::Result<Attributes> {
  let mut attributes = Attributes::new();
  let Some(input) = input else {
    return Ok(attributes);
  };
  for (key, value) in input {
    if key.is_empty() {
      return Err(napi::Error::from_reason(
        "attribute keys must not be empty".to_string(),
      ));
    }
    attributes.insert(attribute_from_key(key), AttributeValue::from(value.clone()));
  }
  Ok(attributes)
}

/// Convert `object_store::Attributes` back into the JS key/value shape, using
/// the canonical name for each fixed attribute.
pub fn convert_attributes(attributes: &Attributes) -> HashMap<String, String> {
  attributes
    .iter()
    .map(|(key, value)| (attribute_key(key), value.to_string()))
    .collect()
}

/// Map a JS attribute key onto an `object_store::Attribute`.
///
/// [`Attribute`] is `#[non_exhaustive]`, so unknown keys are user-defined
/// metadata rather than an error; a future fixed attribute would arrive here as
/// metadata until it is added to this match.
fn attribute_from_key(key: &str) -> Attribute {
  match key.to_ascii_lowercase().as_str() {
    "content-disposition" => Attribute::ContentDisposition,
    "content-encoding" => Attribute::ContentEncoding,
    "content-language" => Attribute::ContentLanguage,
    "content-type" => Attribute::ContentType,
    "cache-control" => Attribute::CacheControl,
    "storage-class" => Attribute::StorageClass,
    _ => Attribute::Metadata(Cow::Owned(key.to_string())),
  }
}

/// The canonical JS name of an `object_store::Attribute`.
fn attribute_key(attribute: &Attribute) -> String {
  match attribute {
    Attribute::ContentDisposition => "content-disposition".to_string(),
    Attribute::ContentEncoding => "content-encoding".to_string(),
    Attribute::ContentLanguage => "content-language".to_string(),
    Attribute::ContentType => "content-type".to_string(),
    Attribute::CacheControl => "cache-control".to_string(),
    Attribute::StorageClass => "storage-class".to_string(),
    Attribute::Metadata(key) => key.to_string(),
    _ => String::new(),
  }
}

/// Build an `object_store::list::PaginatedListOptions` from the JS
/// `PaginatedListOptionsInput` shape.
pub fn build_paginated_options(input: Option<&PaginatedListOptionsInput>) -> RSPaginatedListOptions {
  let Some(input) = input else {
    return RSPaginatedListOptions::default();
  };
  RSPaginatedListOptions {
    offset: input.offset.clone(),
    delimiter: input.delimiter.clone().map(Cow::Owned),
    max_keys: input.max_keys.map(|max| max as usize),
    page_token: input.page_token.clone(),
    extensions: Default::default(),
  }
}
