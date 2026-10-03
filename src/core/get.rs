use std::ops::Range;
use chrono::{DateTime, Utc};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{GetOptions as ObjGetOptions, GetRange, ObjectStoreExt};

use super::types::{ObjectMeta, ObjectStore, RangeInput};

#[napi(object)]
pub struct GetOptions {
  #[napi(ts_type = "{ start?: number; end?: number }")]
  pub range: Option<RangeInput>,
  pub range_start: Option<i64>,
  pub range_end: Option<i64>,
  pub if_match: Option<String>,
  pub if_none_match: Option<String>,
  pub if_modified_since: Option<String>,
  pub if_unmodified_since: Option<String>,
  pub head: Option<bool>,
}

#[napi(object)]
pub struct GetResult {
  pub meta: ObjectMeta,
  pub bytes: Buffer,
}

pub async fn get(store: &ObjectStore, path: String) -> Result<Buffer> {
  let location = Path::from(path.as_str());
  let res = store
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

pub async fn get_result(store: &ObjectStore, path: String) -> Result<GetResult> {
  let location = Path::from(path.as_str());
  let res = store
    .inner
    .get(&location)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  let meta = ObjectMeta::from(res.meta.clone());
  let bytes = res
    .bytes()
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(GetResult {
    meta,
    bytes: Buffer::from(bytes.as_ref()),
  })
}

pub async fn get_opts(
  store: &ObjectStore,
  path: String,
  options: Option<GetOptions>,
) -> Result<Buffer> {
  let location = Path::from(path.as_str());
  let mut get_opts = ObjGetOptions::default();

  if let Some(opts) = options {
    if let Some(r) = opts.range {
      if r.start >= 0 && r.end >= r.start {
        get_opts.range = Some(GetRange::Bounded(Range {
          start: r.start as u64,
          end: r.end as u64,
        }));
      }
    } else if let (Some(start), Some(end)) = (opts.range_start, opts.range_end) {
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
    if let Some(if_modified_since) = opts.if_modified_since {
      if let Ok(dt) = DateTime::parse_from_rfc3339(&if_modified_since) {
        get_opts.if_modified_since = Some(dt.with_timezone(&Utc));
      }
    }
    if let Some(if_unmodified_since) = opts.if_unmodified_since {
      if let Ok(dt) = DateTime::parse_from_rfc3339(&if_unmodified_since) {
        get_opts.if_unmodified_since = Some(dt.with_timezone(&Utc));
      }
    }
    if let Some(head) = opts.head {
      get_opts.head = head;
    }
  }

  let res = store
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

pub async fn get_ranges(
  store: &ObjectStore,
  path: String,
  ranges: Vec<RangeInput>,
) -> Result<Vec<Buffer>> {
  let location = Path::from(path.as_str());
  let rust_ranges: Vec<Range<u64>> = ranges
    .into_iter()
    .map(|r| Range {
      start: r.start.max(0) as u64,
      end: r.end.max(0) as u64,
    })
    .collect();

  let bytes_vec = store
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
