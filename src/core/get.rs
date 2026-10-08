use crate::store::ObjectStore;
use crate::types::{
  build_get_options, convert_attributes, convert_meta, GetOptionsInput, GetResult, Range,
};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{GetResult as RSGetResult, ObjectStoreExt};

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
        .map_err(to_napi_error)?;
      let bytes = res.bytes().await.map_err(to_napi_error)?;
      return Ok(Buffer::from(bytes.as_ref()));
    }

    let res = self.inner.get(&location).await.map_err(to_napi_error)?;
    let bytes = res.bytes().await.map_err(to_napi_error)?;
    Ok(Buffer::from(bytes.as_ref()))
  }

  /// Perform a get request with options
  /// (conditionals, ranges, versioning via `ObjectStore::get_opts`).
  #[napi]
  pub async fn get_opts(&self, path: String, options: GetOptionsInput) -> napi::Result<Buffer> {
    self.get(path, Some(options)).await
  }

  /// Return the bytes and metadata stored at `path`.
  ///
  /// Also reports the byte range that was served (the whole object unless
  /// `options` carried a range) and the attributes stored with the object, which
  /// are read back with [`ObjectStore::put`](crate::store::ObjectStore::put).
  #[napi]
  pub async fn get_with_meta(
    &self,
    path: String,
    options: Option<GetOptionsInput>,
  ) -> napi::Result<GetResult> {
    let location = Path::from(path.as_str());
    let res = match &options {
      Some(opts_input) => {
        let opts = build_get_options(opts_input);
        self
          .inner
          .get_opts(&location, opts)
          .await
          .map_err(to_napi_error)?
      }
      None => self.inner.get(&location).await.map_err(to_napi_error)?,
    };

    split_get_result(res).await
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
      .map_err(to_napi_error)?;
    Ok(Buffer::from(bytes.as_ref()))
  }
}

/// Buffer an [`RSGetResult`], carrying over everything the JS `GetResult`
/// exposes. `RSGetResult::bytes` consumes the result, so the metadata, served
/// range and attributes are read out before the payload is collected.
async fn split_get_result(res: RSGetResult) -> napi::Result<GetResult> {
  let meta = convert_meta(&res.meta);
  let range = Range {
    start: res.range.start as f64,
    end: res.range.end as f64,
  };
  let attributes = convert_attributes(&res.attributes);
  let bytes = res.bytes().await.map_err(to_napi_error)?;
  Ok(GetResult {
    bytes: Buffer::from(bytes.as_ref()),
    meta,
    range,
    attributes,
  })
}

fn to_napi_error(error: object_store::Error) -> napi::Error {
  napi::Error::from_reason(error.to_string())
}
