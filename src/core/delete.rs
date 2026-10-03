use crate::store::ObjectStore;
use crate::types::DeleteOptionsInput;
use futures::StreamExt;
use napi_derive::napi;
use object_store::path::Path;
use object_store::ObjectStoreExt;

#[napi(object)]
pub struct DeleteStreamResult {
  pub path: String,
  pub error: Option<String>,
}

#[napi]
impl ObjectStore {
  #[napi]
  pub async fn delete(
    &self,
    path: String,
    _options: Option<DeleteOptionsInput>,
  ) -> napi::Result<()> {
    let location = Path::from(path.as_str());
    self
      .inner
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
  pub async fn delete_stream(
    &self,
    locations: Vec<String>,
  ) -> napi::Result<Vec<DeleteStreamResult>> {
    let count = locations.len();
    let input = locations.clone();
    let stream = futures::stream::iter(
      locations
        .into_iter()
        .map(|l| Ok::<Path, object_store::Error>(Path::from(l.as_str()))),
    )
    .boxed();

    let mut out = self.inner.delete_stream(stream);
    let mut results: Vec<DeleteStreamResult> = Vec::with_capacity(count);
    while let Some(item) = out.next().await {
      match item {
        Ok(path) => results.push(DeleteStreamResult {
          path: path.to_string(),
          error: None,
        }),
        Err(e) => results.push(DeleteStreamResult {
          path: String::new(),
          error: Some(e.to_string()),
        }),
      }
    }

    for (i, result) in results.iter_mut().enumerate() {
      if result.path.is_empty() {
        if let Some(original) = input.get(i) {
          result.path = original.clone();
        }
      }
    }
    while results.len() < count {
      let path = input[results.len()].clone();
      results.push(DeleteStreamResult {
        path,
        error: Some("bulk delete terminated before this location".to_string()),
      });
    }
    Ok(results)
  }
}
