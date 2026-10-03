use napi_derive::napi;
use object_store::ObjectStoreScheme as RSObjectStoreScheme;
use url::Url;

/// Mirrors `object_store::parse::ObjectStoreScheme` — the kind of object store a
/// URL maps to, as recognised by [`parse_url_scheme`].
#[napi]
pub enum ObjectStoreScheme {
  Local,
  Memory,
  AmazonS3,
  GoogleCloudStorage,
  MicrosoftAzure,
  Http,
}

/// The result of classifying a URL with [`parse_url_scheme`].
#[napi(object)]
pub struct ParsedUrl {
  /// The recognised object store scheme.
  pub scheme: ObjectStoreScheme,
  /// The path into the store, after stripping the scheme/host/bucket.
  pub path: String,
}

/// Classify `url` into an [`ObjectStoreScheme`] and the remaining [`Path`].
///
/// Wraps `object_store::parse::ObjectStoreScheme::parse`. Unlike
/// `ObjectStore::parse_url`, this does not construct a store and requires no
/// cloud features — it only recognises which store a URL would map to.
#[napi]
pub fn parse_url_scheme(url: String) -> napi::Result<ParsedUrl> {
  let parsed = Url::parse(&url).map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let (scheme, path) = RSObjectStoreScheme::parse(&parsed)
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let scheme = match scheme {
    RSObjectStoreScheme::Local => ObjectStoreScheme::Local,
    RSObjectStoreScheme::Memory => ObjectStoreScheme::Memory,
    RSObjectStoreScheme::AmazonS3 => ObjectStoreScheme::AmazonS3,
    RSObjectStoreScheme::GoogleCloudStorage => ObjectStoreScheme::GoogleCloudStorage,
    RSObjectStoreScheme::MicrosoftAzure => ObjectStoreScheme::MicrosoftAzure,
    RSObjectStoreScheme::Http => ObjectStoreScheme::Http,
    // `ObjectStoreScheme` is `#[non_exhaustive]`, so future variants are possible.
    _ => {
      return Err(napi::Error::from_reason(
        "unrecognised ObjectStoreScheme variant".to_string(),
      ))
    }
  };
  Ok(ParsedUrl {
    scheme,
    path: path.as_ref().to_string(),
  })
}
