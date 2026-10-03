pub mod copy;
pub mod delete;
pub mod factory;
pub mod get;
pub mod head;
pub mod list;
pub mod put;
pub mod rename;
pub mod types;

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

pub use copy::CopyOptions;
pub use get::{GetOptions, GetResult};
pub use list::ListResult;
pub use put::{PutOptions, PutResult};
pub use rename::RenameOptions;
pub use types::*;

#[napi]
impl ObjectStore {
  #[napi(constructor)]
  pub fn new(url: Option<String>, options: Option<HashMap<String, String>>) -> Result<Self> {
    factory::new(url, options)
  }

  #[napi(factory)]
  pub fn memory() -> Self {
    factory::memory()
  }

  #[napi(factory)]
  pub fn local(root_path: String) -> Result<Self> {
    factory::local(root_path)
  }

  #[napi]
  pub fn parse_url(
    url: String,
    options: Option<HashMap<String, String>>,
  ) -> Result<ParseUrlResult> {
    factory::parse_url(url, options)
  }

  #[napi]
  pub async fn put(
    &self,
    path: String,
    #[napi(ts_arg_type = "Uint8Array | Buffer")] payload: Uint8Array,
  ) -> Result<PutResult> {
    put::put(self, path, payload).await
  }

  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    #[napi(ts_arg_type = "Uint8Array | Buffer")] payload: Uint8Array,
    options: Option<PutOptions>,
  ) -> Result<PutResult> {
    put::put_opts(self, path, payload, options).await
  }

  #[napi]
  pub async fn get(&self, path: String) -> Result<Buffer> {
    get::get(self, path).await
  }

  #[napi]
  pub async fn get_result(&self, path: String) -> Result<GetResult> {
    get::get_result(self, path).await
  }

  #[napi]
  pub async fn get_opts(&self, path: String, options: Option<GetOptions>) -> Result<Buffer> {
    get::get_opts(self, path, options).await
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<RangeInput>) -> Result<Vec<Buffer>> {
    get::get_ranges(self, path, ranges).await
  }

  #[napi]
  pub async fn head(&self, path: String) -> Result<ObjectMeta> {
    head::head(self, path).await
  }

  #[napi]
  pub async fn delete(&self, path: String) -> Result<()> {
    delete::delete(self, path).await
  }

  #[napi]
  pub async fn list(&self, prefix: Option<String>) -> Result<Vec<ObjectMeta>> {
    list::list(self, prefix).await
  }

  #[napi]
  pub async fn list_with_offset(
    &self,
    prefix: Option<String>,
    offset: String,
  ) -> Result<Vec<ObjectMeta>> {
    list::list_with_offset(self, prefix, offset).await
  }

  #[napi]
  pub async fn copy(&self, from: String, to: String) -> Result<()> {
    copy::copy(self, from, to).await
  }

  #[napi]
  pub async fn copy_opts(
    &self,
    from: String,
    to: String,
    options: Option<CopyOptions>,
  ) -> Result<()> {
    copy::copy_opts(self, from, to, options).await
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> Result<()> {
    rename::rename(self, from, to).await
  }

  #[napi]
  pub async fn rename_opts(
    &self,
    from: String,
    to: String,
    options: Option<RenameOptions>,
  ) -> Result<()> {
    rename::rename_opts(self, from, to, options).await
  }
}

#[napi]
pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParseUrlResult> {
  factory::parse_url(url, options)
}
