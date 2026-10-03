use crate::core::store::ObjectStore;
use crate::core::types::{convert_meta, GetOptionsInput, GetResult};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{GetOptions, ObjectStoreExt};

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn get(&self, path: String) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }

  #[napi]
  pub async fn get_with_meta(&self, path: String) -> napi::Result<GetResult> {
    let location = Path::from(path.as_str());
    let res = self
      .inner
      .get(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    let meta = convert_meta(&res.meta);
    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(GetResult {
      bytes: Buffer::from(bytes.as_ref()),
      meta,
    })
  }

  #[napi]
  pub async fn get_opts(
    &self,
    path: String,
    options: GetOptionsInput,
  ) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
    let mut opts = GetOptions::default();

    if let Some(if_match) = options.if_match {
      opts.if_match = Some(if_match);
    }
    if let Some(if_none_match) = options.if_none_match {
      opts.if_none_match = Some(if_none_match);
    }
    if let (Some(start), Some(end)) = (options.range_start, options.range_end) {
      opts.range = Some(((start as u64)..(end as u64)).into());
    }

    let res = self
      .inner
      .get_opts(&location, opts)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;

    let bytes = res
      .bytes()
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;

    Ok(Buffer::from(bytes.as_ref()))
  }
}
