use version_id::VersionId;

mod base_extractor;
pub use base_extractor::BaseActixVersionIdExtractor;

pub trait ActixVersionIdExtractor: Send + Sync {
    fn extract(
        &self,
        req: &actix_web::HttpRequest,
    ) -> Result<Option<VersionId>, Box<dyn std::error::Error>>;
}
