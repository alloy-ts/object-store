pub mod copy;
pub mod delete;
pub mod factory;
pub mod get;
pub mod get_opts;
pub mod get_ranges;
pub mod head;
pub mod list;
pub mod put;
pub mod put_opts;
pub mod rename;
pub mod types;

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

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
    put_opts::put_opts(self, path, payload, options).await
  }

  #[napi]
  pub async fn get(&self, path: String) -> Result<Buffer> {
    get::get(self, path).await
  }

  #[napi]
  pub async fn get_opts(&self, path: String, options: Option<GetOptions>) -> Result<Buffer> {
    get_opts::get_opts(self, path, options).await
  }

  #[napi]
  pub async fn get_ranges(&self, path: String, ranges: Vec<RangeInput>) -> Result<Vec<Buffer>> {
    get_ranges::get_ranges(self, path, ranges).await
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
  pub async fn copy(&self, from: String, to: String) -> Result<()> {
    copy::copy(self, from, to).await
  }

  #[napi]
  pub async fn rename(&self, from: String, to: String) -> Result<()> {
    rename::rename(self, from, to).await
  }
}

#[napi]
pub fn parse_url(
  url: String,
  options: Option<HashMap<String, String>>,
) -> Result<ParseUrlResult> {
  factory::parse_url(url, options)
}
