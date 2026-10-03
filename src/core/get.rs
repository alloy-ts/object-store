use crate::store::ObjectStore;
use crate::types::{build_get_options, convert_meta, GetOptionsInput, GetResult, Range};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi]
impl ObjectStore {
  /// Return the bytes stored at `path`.
  ///
  /// Wraps `ObjectStoreExt::get`. Passing `options` routes through
  /// `ObjectStore::get_opts` (conditional gets).
  #[napi]
  pub async fn get(
    &self,
    path: String,
    options: Option<GetOptionsInput>,
  ) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());

    if let Some(opts_input) = options {
      let opts = build_get_options(&opts_input);
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

  /// Perform a get request with options
  /// (conditionals, ranges, versioning via `ObjectStore::get_opts`).
  #[napi]
  pub async fn get_opts(&self, path: String, options: GetOptionsInput) -> napi::Result<Buffer> {
    self.get(path, Some(options)).await
  }

  /// Return the bytes and metadata stored at `path`.
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

  /// Return the bytes stored at `path` within `range` (`{ start, end }`,
  /// end exclusive).
  ///
  /// Wraps `ObjectStoreExt::get_range`.
  #[napi]
  pub async fn get_range(&self, path: String, range: Range) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
    let bytes = self
      .inner
      .get_range(&location, (range.start as u64)..(range.end as u64))
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes.as_ref()))
  }
}
