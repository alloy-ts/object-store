use napi_derive::napi;
use object_store::TagSet as RsTagSet;

/// A collection of key value pairs used to annotate objects.
///
/// https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-tagging.html
/// https://learn.microsoft.com/en-us/rest/api/storageservices/set-blob-tags
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[napi]
pub struct TagSet {
  pub(crate) inner: RsTagSet,
}

#[napi]
impl TagSet {
  /// Create a new, empty TagSet.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsTagSet::default(),
    }
  }

  /// Append a key value pair to this TagSet.
  ///
  /// Stores have different restrictions on what characters are permitted,
  /// for portability it is recommended applications use no more than 10 tags,
  /// and stick to alphanumeric characters, and + - = . _ : /
  #[napi]
  pub fn push(&mut self, key: String, value: String) {
    self.inner.push(&key, &value);
  }

  /// Return this TagSet as a URL-encoded string.
  #[napi]
  pub fn encoded(&self) -> String {
    self.inner.encoded().to_string()
  }

  /// Return whether this TagSet contains any tags.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tag_set() {
    let mut set = TagSet::default();
    set.push("test/foo".to_string(), "value sdlks".to_string());
    set.push("foo".to_string(), " sdf _ /+./sd".to_string());
    assert_eq!(
      set.encoded(),
      "test%2Ffoo=value+sdlks&foo=+sdf+_+%2F%2B.%2Fsd"
    );
  }
}
