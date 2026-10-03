use napi_derive::napi;

#[napi(object)]
pub struct EnabledFeatures {
  pub fs: bool,
  pub tokio: bool,
  pub aws: bool,
  pub azure: bool,
  pub gcp: bool,
  pub http: bool,
}

#[napi]
pub fn get_enabled_features() -> EnabledFeatures {
  EnabledFeatures {
    fs: cfg!(feature = "fs"),
    tokio: true,
    aws: cfg!(feature = "aws"),
    azure: cfg!(feature = "azure"),
    gcp: cfg!(feature = "gcp"),
    http: cfg!(feature = "http"),
  }
}
