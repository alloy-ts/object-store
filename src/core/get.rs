use crate::core::store::ObjectStore;
use crate::core::types::{convert_meta, GetOptionsInput, GetResult};
use chrono::DateTime;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{GetOptions, GetRange, ObjectStoreExt};

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn get(
    &self,
    path: String,
    options: Option<GetOptionsInput>,
  ) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());

    if let Some(opts_input) = options {
      let mut opts = GetOptions::default();
      if let Some(if_match) = opts_input.if_match {
        opts.if_match = Some(if_match);
      }
      if let Some(if_none_match) = opts_input.if_none_match {
        opts.if_none_match = Some(if_none_match);
      }
      if let Some(ms) = opts_input.if_modified_since {
        if let Some(dt) = DateTime::from_timestamp_millis(ms) {
          opts.if_modified_since = Some(dt);
        }
      }
      if let Some(ms) = opts_input.if_unmodified_since {
        if let Some(dt) = DateTime::from_timestamp_millis(ms) {
          opts.if_unmodified_since = Some(dt);
        }
      }
      if let Some(r) = opts_input.range {
        if let (Some(start), Some(end)) = (r.start, r.end) {
          opts.range = Some(GetRange::Bounded((start as u64)..(end as u64)));
        } else if let Some(offset) = r.offset {
          opts.range = Some(GetRange::Offset(offset as u64));
        } else if let Some(suffix) = r.suffix {
          opts.range = Some(GetRange::Suffix(suffix as u64));
        }
      }
      if let Some(v) = opts_input.version {
        opts.version = Some(v);
      }
      if let Some(h) = opts_input.head {
        opts.head = h;
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
      return Ok(Buffer::from(bytes.as_ref()));
    }

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
  pub async fn get_opts(&self, path: String, options: GetOptionsInput) -> napi::Result<Buffer> {
    self.get(path, Some(options)).await
  }
}
