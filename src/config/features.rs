use napi_derive::napi;

#[napi(object)]
pub struct FeatureFlags {
  pub fs: bool,
  pub tokio: bool,
  pub aws: bool,
  pub azure: bool,
  pub gcp: bool,
  pub http: bool,
}

#[napi]
pub fn get_features() -> FeatureFlags {
  FeatureFlags {
    fs: cfg!(feature = "fs"),
    tokio: cfg!(feature = "tokio"),
    aws: cfg!(feature = "aws"),
    azure: cfg!(feature = "azure"),
    gcp: cfg!(feature = "gcp"),
    http: cfg!(feature = "http"),
  }
}
