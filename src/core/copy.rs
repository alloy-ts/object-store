use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::{CopyOptions, ObjectStore};

pub async fn store_copy(store: &ObjectStore, from: String, to: String) -> Result<()> {
  let from_p = Path::from(from.as_str());
  let to_p = Path::from(to.as_str());
  store
    .inner
    .copy(&from_p, &to_p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}

pub async fn store_copy_opts(
  store: &ObjectStore,
  from: String,
  to: String,
  options: CopyOptions,
) -> Result<()> {
  let from_p = Path::from(from.as_str());
  let to_p = Path::from(to.as_str());
  let mut opts = object_store::CopyOptions::default();
  if let Some(m) = options.mode {
    if m == "create" {
      opts.mode = object_store::CopyMode::Create;
    }
  }
  store
    .inner
    .copy_opts(&from_p, &to_p, opts)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
