use std::ops::Range;
use napi::bindgen_prelude::*;
use object_store::path::Path;

use super::types::{ObjectStore, RangeInput};

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
