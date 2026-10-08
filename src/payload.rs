use bytes::Bytes;
use napi::bindgen_prelude::{Buffer, Either};
use napi_derive::napi;
use object_store::PutPayload as RsPutPayload;
use object_store::PutPayloadMut as RsPutPayloadMut;

#[derive(Clone)]
#[napi]
pub struct PutPayload {
  pub(crate) inner: RsPutPayload,
}

#[napi]
impl PutPayload {
  /// Create a new empty `PutPayload`.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsPutPayload::new(),
    }
  }

  /// Build a `PutPayload` from a single buffer (copied into a `Bytes`).
  #[napi(factory)]
  pub fn from_bytes(data: Buffer) -> Self {
    Self {
      inner: RsPutPayload::from(Bytes::from(data.to_vec())),
    }
  }

  /// Build a `PutPayload` from a UTF-8 string.
  #[napi(factory)]
  pub fn from_string(data: String) -> Self {
    Self {
      inner: RsPutPayload::from(data),
    }
  }

  /// Total number of bytes across all chunks.
  #[napi]
  pub fn content_length(&self) -> f64 {
    self.inner.content_length() as f64
  }

  /// Whether this payload holds no bytes.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.content_length() == 0
  }

  /// Return each underlying chunk as a separate `Buffer`.
  ///
  /// A `PutPayload` is an ordered collection of `Bytes`; this exposes the
  /// individual chunks without reallocating.
  #[napi]
  pub fn chunks(&self) -> Vec<Buffer> {
    self
      .inner
      .as_ref()
      .iter()
      .map(|b| Buffer::from(b.as_ref()))
      .collect()
  }

  /// Concatenate every chunk into a single `Buffer`.
  #[napi]
  pub fn concat(&self) -> Buffer {
    Buffer::from(Bytes::from(self.inner.clone()).as_ref())
  }

  /// Cheaply clone the payload (shares the underlying `Bytes` via `Arc`).
  #[napi]
  pub fn clone(&self) -> PutPayload {
    PutPayload {
      inner: self.inner.clone(),
    }
  }
}

/// NAPI binding for `object_store::PutPayloadMut`.
///
/// A builder for [`PutPayload`] that avoids reallocating memory: data is
/// accumulated in fixed blocks (default 8 KiB), flushed to `Bytes` once full.
/// Call `freeze` to obtain an immutable [`PutPayload`] (which is what `put` /
/// `putPart` accept).
#[napi]
pub struct PutPayloadMut {
  pub(crate) inner: RsPutPayloadMut,
}

#[napi]
impl PutPayloadMut {
  /// Create a new `PutPayloadMut` with the default 8 KiB block size.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RsPutPayloadMut::new(),
    }
  }

  /// Create a `PutPayloadMut` with a custom minimum allocation block size.
  ///
  /// Must be called before any data is written.
  #[napi(factory)]
  pub fn with_block_size(block_size: u32) -> Self {
    Self {
      inner: RsPutPayloadMut::new().with_block_size(block_size as usize),
    }
  }

  /// Append `data` as a new `Bytes` chunk without copying the underlying data
  /// again. Closes any in-progress block first.
  #[napi]
  pub fn push(&mut self, data: Buffer) {
    self.inner.push(Bytes::from(data.to_vec()));
  }

  /// Write `data` into this payload using the block-buffered allocator.
  #[napi]
  pub fn extend_from_slice(&mut self, data: Buffer) {
    self.inner.extend_from_slice(&data.to_vec());
  }

  /// Total number of bytes written so far.
  #[napi]
  pub fn content_length(&self) -> f64 {
    self.inner.content_length() as f64
  }

  /// Whether no bytes have been written.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.content_length() == 0
  }

  /// Freeze into an immutable [`PutPayload`]. This instance becomes empty.
  #[napi]
  pub fn freeze(&mut self) -> PutPayload {
    let mut tmp = RsPutPayloadMut::new();
    std::mem::swap(&mut self.inner, &mut tmp);
    PutPayload {
      inner: tmp.freeze(),
    }
  }
}

/// Accept either a raw `Buffer` or a `PutPayload` as a `put` payload.
///
/// Shared by every `put` / `putPart` binding so the conversion lives in one
/// place. (A `PutPayloadMut` must be `freeze`d to a `PutPayload` first.)
pub(crate) fn to_put_payload(data: Either<Buffer, &PutPayload>) -> RsPutPayload {
  match data {
    Either::A(buf) => RsPutPayload::from(Bytes::from(buf.to_vec())),
    Either::B(p) => p.inner.clone(),
  }
}
