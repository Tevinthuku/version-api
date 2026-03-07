use version_id::VersionId;

mod base_extractor;
pub use base_extractor::BaseActixVersionIdExtractor;

// This trait lets applications define where API version IDs come from.
// The version does not have to be in headers/query/path; it can be resolved
// earlier (e.g., middleware loads an account's pinned API version from the DB) and stored
// in request extensions for the extractor to read.
pub trait ActixVersionIdExtractor: Send + Sync {
    fn extract(
        &self,
        req: &actix_web::HttpRequest,
    ) -> Result<Option<VersionId>, Box<dyn std::error::Error>>;
}
