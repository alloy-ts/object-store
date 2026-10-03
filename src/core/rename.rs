use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::{ObjectStore, RenameOptionsParam};

pub async fn store_rename(store: &ObjectStore, from: String, to: String) -> Result<()> {
  let from_p = Path::from(from.as_str());
  let to_p = Path::from(to.as_str());
  store
    .inner
    .rename(&from_p, &to_p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}

pub async fn store_rename_opts(
  store: &ObjectStore,
  from: String,
  to: String,
  options: RenameOptionsParam,
) -> Result<()> {
  let from_p = Path::from(from.as_str());
  let to_p = Path::from(to.as_str());
  let mut opts = object_store::RenameOptions::default();
  if let Some(m) = options.mode {
    if m == "create" {
      opts.target_mode = object_store::RenameTargetMode::Create;
    }
  }
  store
    .inner
    .rename_opts(&from_p, &to_p, opts)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(())
}
