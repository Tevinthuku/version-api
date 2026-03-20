use version_core::ApiVersionId;

#[derive(ApiVersionId)]
pub enum ApiVersion {
    #[version("2.0.0")]
    V2_0_0,
    #[version("1.0.0")]
    V1_0_0,
    #[version("0.5.0")]
    V0_5_0,
}
