use crate::store::ObjectStore;
use crate::types::Range;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<Range>) -> napi::Result<Vec<Buffer>> {
    let location = Path::from(path.as_str());
    let range_ops: Vec<std::ops::Range<u64>> = ranges
      .iter()
      .map(|r| (r.start as u64)..(r.end as u64))
      .collect();
    let results = self
      .inner
      .get_ranges(&location, &range_ops)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(
      results
        .into_iter()
        .map(|b| Buffer::from(b.as_ref()))
        .collect(),
    )
  }
}
