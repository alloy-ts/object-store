use crate::store::ObjectStore;
use crate::types::{
  build_paginated_options, convert_meta, ListOptionsInput, ListResult, ObjectMeta,
  PaginatedListOptionsInput, PaginatedListResult,
};
use async_trait::async_trait;
use futures::StreamExt;
use http::Extensions;
use napi_derive::napi;
use object_store::list::{
  PaginatedListOptions, PaginatedListResult as RSPaginatedListResult, PaginatedListStore,
};
use object_store::path::Path;
use object_store::{ListResult as RSListResult, ObjectMeta as RSObjectMeta};
use object_store::ObjectStore as ObjectStoreTrait;
use std::sync::Arc;

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn list(
    &self,
    prefix: Option<String>,
    _options: Option<ListOptionsInput>,
  ) -> napi::Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let mut stream = self.inner.list(prefix_path.as_ref());
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

  #[napi]
  pub async fn list_with_delimiter(&self, prefix: Option<String>) -> napi::Result<ListResult> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let res = self
      .inner
      .list_with_delimiter(prefix_path.as_ref())
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(ListResult {
      objects: res.objects.iter().map(convert_meta).collect(),
      common_prefixes: res.common_prefixes.iter().map(|p| p.to_string()).collect(),
    })
  }
  #[napi]
  pub async fn list_with_offset(
    &self,
    prefix: Option<String>,
    offset: String,
  ) -> napi::Result<Vec<ObjectMeta>> {
    let prefix_path = prefix.map(|p| Path::from(p.as_str()));
    let offset_path = Path::from(offset.as_str());
    let mut stream = self
      .inner
      .list_with_offset(prefix_path.as_ref(), &offset_path);
    let mut results = Vec::new();
    while let Some(item) = stream.next().await {
      let meta = item.map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
      results.push(convert_meta(&meta));
    }
    Ok(results)
  }

  /// Perform a paginated list request.
  ///
  /// Wraps `object_store::list::PaginatedListStore::list_paginated`. Note that,
  /// as for the underlying trait, the order of returned objects is not
  /// guaranteed and — unlike [`ObjectStore::list_with_delimiter`] — a trailing
  /// delimiter is not automatically added to `prefix`.
  #[napi]
  pub async fn list_paginated(
    &self,
    prefix: Option<String>,
    options: Option<PaginatedListOptionsInput>,
  ) -> napi::Result<PaginatedListResult> {
    let opts = build_paginated_options(options.as_ref());
    let store = EmulatedPaginatedStore {
      inner: self.inner.clone(),
    };
    let res = store
      .list_paginated(prefix.as_deref(), opts)
      .await
      .map_err(|e: object_store::Error| napi::Error::from_reason(e.to_string()))?;
    Ok(PaginatedListResult {
      result: ListResult {
        objects: res.result.objects.iter().map(convert_meta).collect(),
        common_prefixes: res
          .result
          .common_prefixes
          .iter()
          .map(|p| p.to_string())
          .collect(),
      },
      page_token: res.page_token,
    })
  }
}

/// Prefix of the page tokens minted by [`EmulatedPaginatedStore`].
///
/// Tokens are opaque to callers, but stamping them lets a token that did not
/// come from a previous `listPaginated` call be rejected instead of being
/// silently treated as a path to resume from.
const PAGE_TOKEN_PREFIX: &str = "object-store:napi:paginated:v1:";

/// A single row of a paginated listing: either an object or a common prefix.
enum PaginatedEntry {
  Object(RSObjectMeta),
  Prefix(Path),
}

impl PaginatedEntry {
  /// The key this entry sorts and resumes on, i.e. what a page token encodes.
  fn key(&self) -> String {
    match self {
      PaginatedEntry::Object(meta) => meta.location.to_string(),
      PaginatedEntry::Prefix(prefix) => prefix.to_string(),
    }
  }
}

/// [`PaginatedListStore`] synthesised on top of [`ObjectStoreTrait`] for
/// backends that do not implement the trait themselves, such as
/// `InMemory` and `LocalFileSystem`.
///
/// The trait exists so that stores backed by a paginated API (currently only
/// AWS, GCP and Azure in `object_store`, none of which are enabled in this
/// crate) can hand out an opaque continuation token. For the remaining
/// backends a token cannot be produced by the store, so it is derived from the
/// last key of the page: the next page is simply the listing that follows it.
///
/// [`PaginatedListStore`]: object_store::list::PaginatedListStore
/// [`ObjectStoreTrait`]: object_store::ObjectStore
struct EmulatedPaginatedStore {
  inner: Arc<dyn ObjectStoreTrait>,
}

#[async_trait]
impl PaginatedListStore for EmulatedPaginatedStore {
  async fn list_paginated(
    &self,
    prefix: Option<&str>,
    opts: PaginatedListOptions,
  ) -> object_store::Result<RSPaginatedListResult> {
    let prefix_path = prefix.map(Path::from);
    // A page token already encodes where to resume, so it takes precedence
    // over `offset`.
    let start_after = match opts.page_token.as_deref() {
      Some(token) => Some(decode_page_token(token)?),
      None => opts.offset.clone(),
    };
    // Some stores only support `/`; any other delimiter is passed through as
    // the "use a delimiter" signal and grouping is left to the store.
    let delimited = opts.delimiter.is_some();

    let mut entries = Vec::new();
    let extensions;
    if delimited {
      let res = self
        .inner
        .list_with_delimiter(prefix_path.as_ref())
        .await?;
      entries.extend(res.objects.into_iter().map(PaginatedEntry::Object));
      entries.extend(res.common_prefixes.into_iter().map(PaginatedEntry::Prefix));
      if let Some(start) = start_after.as_deref() {
        entries.retain(|entry| entry.key().as_str() > start);
      }
      // Objects and common prefixes arrive as two separate lists, so interleave
      // them to keep page tokens monotonic.
      entries.sort_by_key(PaginatedEntry::key);
      extensions = res.extensions;
    } else {
      let mut stream = match start_after.as_deref() {
        // `list_with_offset` omits the object at the offset.
        Some(start) => self
          .inner
          .list_with_offset(prefix_path.as_ref(), &Path::from(start)),
        None => self.inner.list(prefix_path.as_ref()),
      };
      while let Some(item) = stream.next().await {
        entries.push(PaginatedEntry::Object(item?));
      }
      extensions = Extensions::new();
    }

    // Only hand out a token when entries were actually dropped, so that the
    // last page of a listing is reported as the final one.
    let truncated = opts.max_keys.is_some_and(|max| entries.len() > max);
    if let Some(max) = opts.max_keys {
      entries.truncate(max);
    }
    let page_token = truncated
      .then(|| entries.last().map(PaginatedEntry::key))
      .flatten()
      .map(|key| format!("{PAGE_TOKEN_PREFIX}{key}"));

    let mut objects = Vec::new();
    let mut common_prefixes = Vec::new();
    for entry in entries {
      match entry {
        PaginatedEntry::Object(meta) => objects.push(meta),
        PaginatedEntry::Prefix(prefix) => common_prefixes.push(prefix),
      }
    }
    Ok(RSPaginatedListResult {
      result: RSListResult {
        common_prefixes,
        objects,
        extensions,
      },
      page_token,
    })
  }
}

/// Recover the key a page token was minted from.
fn decode_page_token(token: &str) -> object_store::Result<String> {
  token.strip_prefix(PAGE_TOKEN_PREFIX).map(str::to_string).ok_or_else(|| {
    invalid_input(format!(
      "malformed page token {token:?}: expected a token returned by listPaginated"
    ))
  })
}

fn invalid_input(message: String) -> object_store::Error {
  object_store::Error::Generic {
    store: "PaginatedList",
    source: Box::new(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      message,
    )),
  }
}
