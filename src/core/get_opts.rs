use std::ops::Range;
use napi::bindgen_prelude::*;
use object_store::path::Path;
use object_store::{GetOptions as ObjGetOptions, GetRange};

use super::types::{GetOptions, ObjectStore};

pub async fn get_opts(
  store: &ObjectStore,
  path: String,
  options: Option<GetOptions>,
) -> Result<Buffer> {
  let location = Path::from(path.as_str());
  let mut get_opts = ObjGetOptions::default();

  if let Some(opts) = options {
    if let (Some(start), Some(end)) = (opts.range_start, opts.range_end) {
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
