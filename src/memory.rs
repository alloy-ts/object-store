use crate::types::{
  build_get_options, build_put_options, convert_meta, CopyOptionsInput, DeleteOptionsInput,
  GetOptionsInput, GetResult, HeadOptionsInput, ListOptionsInput, ListResult, ObjectMeta,
  PutOptionsInput, PutResult, Range, RenameOptionsInput,
};
use bytes::Bytes;
use futures::StreamExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use object_store::memory::InMemory as RSInMemory;
use object_store::path::Path;
use object_store::{ObjectStore as ObjectStoreTrait, ObjectStoreExt, PutPayload};
use std::sync::Arc;

/// In-memory storage suitable for testing or for opting out of using a cloud
/// storage provider.
///
/// Wraps `object_store::memory::InMemory`. Unlike the generic [`crate::store::ObjectStore`]
/// (which erases the concrete type behind `dyn ObjectStore`), this binding keeps the
/// concrete `InMemory` type so it can expose [`InMemory::fork`], which snapshots the
/// current contents into a brand-new store.
#[napi]
pub struct InMemory {
  pub(crate) inner: Arc<RSInMemory>,
}

#[napi]
impl InMemory {
  /// Create new in-memory storage.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Arc::new(RSInMemory::new()),
    }
  }

  /// Creates a fork of the store, with the current content copied into the new store.
  #[napi]
  pub fn fork(&self) -> InMemory {
    let forked = self.inner.fork();
    InMemory {
      inner: Arc::new(forked),
    }
  }

  #[napi]
  pub async fn put(
    &self,
    path: String,
    data: Buffer,
    options: Option<PutOptionsInput>,
  ) -> napi::Result<PutResult> {
    let location = Path::from(path.as_str());
    let payload = PutPayload::from(Bytes::from(data.to_vec()));
    let store = self.inner.as_ref();
    let res = match options {
      Some(opts) => store
        .put_opts(&location, payload, build_put_options(&opts))
        .await,
      None => store.put(&location, payload).await,
    }
    .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PutResult {
      e_tag: res.e_tag,
      version: res.version,
    })
  }

  #[napi]
  pub async fn put_opts(
    &self,
    path: String,
    data: Buffer,
    options: PutOptionsInput,
  ) -> napi::Result<PutResult> {
    self.put(path, data, Some(options)).await
  }

  #[napi]
  pub async fn get(&self, path: String, options: Option<GetOptionsInput>) -> napi::Result<Buffer> {
    let location = Path::from(path.as_str());
    let store = self.inner.as_ref();
    let res = match options {
      Some(opts) => store.get_opts(&location, build_get_options(&opts)).await,
      None => store.get(&location).await,
    }
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
    let store = self.inner.as_ref();
    let res = store
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

  #[napi]
  pub async fn head(
    &self,
    path: String,
    _options: Option<HeadOptionsInput>,
  ) -> napi::Result<ObjectMeta> {
    let location = Path::from(path.as_str());
    let store = self.inner.as_ref();
    let meta = store
      .head(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(convert_meta(&meta))
  }

  #[napi]
  pub async fn head_opts(
    &self,
    path: String,
    options: Option<HeadOptionsInput>,
  ) -> napi::Result<ObjectMeta> {
    self.head(path, options).await
  }

  #[napi]
  pub async fn list(
    &self,
    prefix: Option<String>,
    _options: Option<ListOptionsInput>,
  ) -> napi::Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let store = self.inner.as_ref();
    let mut stream = store.list(prefix_path.as_ref());
    let mut results = Vec::new();
    while let Some(item) = stream.next().await {
      let meta = item.map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      results.push(convert_meta(&meta));
    }
    Ok(results)
  }

  #[napi]
  pub async fn list_opts(
    &self,
    prefix: Option<String>,
    options: Option<ListOptionsInput>,
  ) -> napi::Result<Vec<ObjectMeta>> {
    self.list(prefix, options).await
  }

  /// The memory implementation returns all results, as opposed to the cloud
  /// versions which limit their results to 1k or more because of API limitations.
  #[napi]
  pub async fn list_with_delimiter(
    &self,
    prefix: Option<String>,
  ) -> napi::Result<ListResult> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let store = self.inner.as_ref();
    let res = store
      .list_with_delimiter(prefix_path.as_ref())
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(ListResult {
      objects: res.objects.iter().map(convert_meta).collect(),
      common_prefixes: res.common_prefixes.iter().map(|p| p.to_string()).collect(),
    })
  }

  #[napi]
  pub async fn delete(
    &self,
    path: String,
    _options: Option<DeleteOptionsInput>,
  ) -> napi::Result<()> {
    let location = Path::from(path.as_str());
    let store = self.inner.as_ref();
    store
      .delete(&location)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub async fn delete_opts(
    &self,
    path: String,
    options: Option<DeleteOptionsInput>,
  ) -> napi::Result<()> {
    self.delete(path, options).await
  }

  #[napi]
  pub async fn copy(
    &self,
    from: String,
    to: String,
    options: Option<CopyOptionsInput>,
  ) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    let store = self.inner.as_ref();
    if options.and_then(|o| o.if_not_exists) == Some(true) {
      store
        .copy_if_not_exists(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    } else {
      store
        .copy(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    }
    Ok(())
  }

  #[napi]
  pub async fn copy_opts(
    &self,
    from: String,
    to: String,
    options: Option<CopyOptionsInput>,
  ) -> napi::Result<()> {
    self.copy(from, to, options).await
  }

  #[napi]
  pub async fn rename(
    &self,
    from: String,
    to: String,
    options: Option<RenameOptionsInput>,
  ) -> napi::Result<()> {
    let from_path = Path::from(from.as_str());
    let to_path = Path::from(to.as_str());
    let store = self.inner.as_ref();
    if options.and_then(|o| o.target_mode_create) == Some(true) {
      store
        .rename_if_not_exists(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    } else {
      store
        .rename(&from_path, &to_path)
        .await
        .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    }
    Ok(())
  }

  #[napi]
  pub async fn rename_opts(
    &self,
    from: String,
    to: String,
    options: Option<RenameOptionsInput>,
  ) -> napi::Result<()> {
    self.rename(from, to, options).await
  }

  #[napi]
  pub async fn get_ranges(
    &self,
    path: String,
    ranges: Vec<Range>,
  ) -> napi::Result<Vec<Buffer>> {
    let location = Path::from(path.as_str());
    let range_ops: Vec<std::ops::Range<u64>> = ranges
      .iter()
      .map(|r| (r.start as u64)..(r.end as u64))
      .collect();
    let store = self.inner.as_ref();
    let results = store
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
