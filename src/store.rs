use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use napi_derive::napi;
use object_store::path::Path;
use object_store::{
  local::LocalFileSystem, memory::InMemory, GetOptions, GetResult, ListResult, MultipartUpload,
  ObjectMeta, ObjectStore as ObjectStoreTrait, PutMultipartOptions, PutOptions, PutPayload,
  PutResult, Result,
};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use url::Url;

#[napi]
pub struct ObjectStore {
  pub(crate) inner: Arc<dyn ObjectStoreTrait>,
}

#[derive(Debug)]
pub struct Wrapper {
  pub inner: Arc<dyn ObjectStoreTrait>,
}

impl std::fmt::Display for Wrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Wrapper({})", self.inner)
  }
}

#[async_trait]
#[deny(clippy::missing_trait_methods)]
impl ObjectStoreTrait for Wrapper {
  async fn put_opts(
    &self,
    location: &Path,
    payload: PutPayload,
    opts: PutOptions,
  ) -> Result<PutResult> {
    self.inner.put_opts(location, payload, opts).await
  }

  async fn put_multipart_opts(
    &self,
    location: &Path,
    opts: PutMultipartOptions,
  ) -> Result<Box<dyn MultipartUpload>> {
    self.inner.put_multipart_opts(location, opts).await
  }

  async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult> {
    self.inner.get_opts(location, options).await
  }

  async fn get_ranges(
    &self,
    location: &Path,
    ranges: &[Range<u64>],
  ) -> Result<Vec<Bytes>> {
    self.inner.get_ranges(location, ranges).await
  }

  fn delete_stream(
    &self,
    locations: BoxStream<'static, Result<Path>>,
  ) -> BoxStream<'static, Result<Path>> {
    self.inner.delete_stream(locations)
  }

  fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
    self.inner.list(prefix)
  }

  fn list_with_offset(
    &self,
    prefix: Option<&Path>,
    offset: &Path,
  ) -> BoxStream<'static, Result<ObjectMeta>> {
    self.inner.list_with_offset(prefix, offset)
  }

  async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
    self.inner.list_with_delimiter(prefix).await
  }

  async fn copy_opts(
    &self,
    from: &Path,
    to: &Path,
    options: object_store::CopyOptions,
  ) -> Result<()> {
    self.inner.copy_opts(from, to, options).await
  }

  async fn rename_opts(
    &self,
    from: &Path,
    to: &Path,
    options: object_store::RenameOptions,
  ) -> Result<()> {
    self.inner.rename_opts(from, to, options).await
  }
}

#[napi]
impl ObjectStore {
  #[napi(factory)]
  pub fn create_in_memory() -> Self {
    Self {
      inner: Arc::new(InMemory::new()),
    }
  }

  #[napi(factory)]
  pub fn create_local(root_path: String) -> napi::Result<Self> {
    let local = LocalFileSystem::new_with_prefix(root_path)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner: Arc::new(local),
    })
  }

  #[napi(factory)]
  pub fn parse_url(url: String, options: Option<HashMap<String, String>>) -> napi::Result<Self> {
    let _ = options; // local/memory stores take no URL options
    let parsed_url =
      Url::parse(&url).map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
    let store: Arc<dyn ObjectStoreTrait> = match parsed_url.scheme() {
      "file" => Arc::new(
        LocalFileSystem::new_with_prefix(parsed_url.path())
          .map_err(|e| napi::Error::from_reason(e.to_string()))?,
      ),
      "memory" => Arc::new(InMemory::new()),
      scheme => {
        return Err(napi::Error::from_reason(format!(
          "unsupported scheme '{scheme}': only file:// and memory:// are supported \
           (cloud/reqwest features are disabled; http-base requires a custom HttpConnector \
           to use http(s):// URLs)"
        )))
      }
    };
    Ok(Self { inner: store })
  }
}
