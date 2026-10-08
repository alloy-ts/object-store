use napi_derive::napi;
use object_store::TagSet as RsTagSet;

/// NAPI binding for `object_store::TagSet`.
///
/// A collection of key-value pairs used to annotate objects.
#[napi]
pub struct TagSet {
  pub(crate) inner: RsTagSet,
}

#[napi]
impl TagSet {
  /// Create a new empty `TagSet`.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsTagSet::default(),
    }
  }

  /// Append a key-value pair to this `TagSet`.
  #[napi]
  pub fn push(&mut self, key: String, value: String) {
    self.inner.push(&key, &value);
  }

  /// Return this `TagSet` as a URL-encoded string.
  #[napi]
  pub fn encoded(&self) -> String {
    self.inner.encoded().to_string()
  }

  /// Return whether this `TagSet` contains any tags.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}
