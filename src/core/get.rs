use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::ObjectStoreExt;

use crate::core::types::{convert_meta, GetOptionsParam, GetResult, ObjectStore, RangeParam};

pub async fn store_get(store: &ObjectStore, path: String) -> Result<GetResult> {
  let p = Path::from(path.as_str());
  let res = store
    .inner
    .get(&p)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let meta = convert_meta(&res.meta);
  let bytes = res
    .bytes()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(GetResult {
    bytes: Buffer::from(bytes.to_vec()),
    meta,
  })
}

pub async fn store_get_opts(
  store: &ObjectStore,
  path: String,
  options: GetOptionsParam,
) -> Result<GetResult> {
  let p = Path::from(path.as_str());
  let mut opts = object_store::GetOptions::default();
  if let Some(m) = options.if_match {
    opts.if_match = Some(m);
  }
  if let Some(nm) = options.if_none_match {
    opts.if_none_match = Some(nm);
  }
  if let Some(r) = options.range {
    opts.range = Some(object_store::GetRange::Bounded(
      (r.start as u64)..(r.start as u64 + r.length as u64),
    ));
  }
  let res = store
    .inner
    .get_opts(&p, opts)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let meta = convert_meta(&res.meta);
  let bytes = res
    .bytes()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(GetResult {
    bytes: Buffer::from(bytes.to_vec()),
    meta,
  })
}

pub async fn store_get_range(
  store: &ObjectStore,
  path: String,
  start: i64,
  length: i64,
) -> Result<Buffer> {
  let p = Path::from(path.as_str());
  let range = (start as u64)..(start as u64 + length as u64);
  let bytes = store
    .inner
    .get_range(&p, range)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(Buffer::from(bytes.to_vec()))
}

pub async fn store_get_ranges(
  store: &ObjectStore,
  path: String,
  ranges: Vec<RangeParam>,
) -> Result<Vec<Buffer>> {
  let p = Path::from(path.as_str());
  let rs: Vec<std::ops::Range<u64>> = ranges
    .iter()
    .map(|r| (r.start as u64)..(r.start as u64 + r.length as u64))
    .collect();
  let results = store
    .inner
    .get_ranges(&p, &rs)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(results.into_iter().map(|b| Buffer::from(b.to_vec())).collect())
}
