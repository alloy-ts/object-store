use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{
  ObjectStoreExt, PutMode, PutOptions as ObjPutOptions, PutPayload, TagSet as ObjTagSet, UpdateVersion,
};

use super::types::ObjectStore;

#[napi(object)]
pub struct PutResult {
  pub e_tag: Option<String>,
  pub version: Option<String>,
}

#[napi(object)]
pub struct PutOptions {
  #[napi(ts_type = "'overwrite' | 'create' | 'update'")]
  pub mode: Option<String>,
  pub e_tag: Option<String>,
  pub version: Option<String>,
  pub tags: Option<HashMap<String, String>>,
}

pub async fn put(store: &ObjectStore, path: String, payload: Uint8Array) -> Result<PutResult> {
  let location = Path::from(path.as_str());
  let put_payload = PutPayload::from(payload.to_vec());
  let res = store
    .inner
    .put(&location, put_payload)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}

pub async fn put_opts(
  store: &ObjectStore,
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

  let mut put_options = ObjPutOptions::from(put_mode);

  if let Some(opts) = options {
    if let Some(tags) = opts.tags {
      let mut tag_set = ObjTagSet::default();
      for (k, v) in tags {
        tag_set.push(&k, &v);
      }
      put_options.tags = tag_set;
    }
  }

  let res = store
    .inner
    .put_opts(&location, put_payload, put_options)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;

  Ok(PutResult {
    e_tag: res.e_tag,
    version: res.version,
  })
}
