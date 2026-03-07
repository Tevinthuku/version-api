use actix_web::http::header::HeaderMap;
use version_id::{VersionId, VersionIdExtractor};

pub struct VersionIdHeaderExtractor {
    header_name: String,
}

impl VersionIdHeaderExtractor {
    pub fn new(header_name: String) -> Self {
        Self { header_name }
    }
}

impl VersionIdExtractor for VersionIdHeaderExtractor {
    type Input = HeaderMap;

    fn extract(
        &self,
        input: &Self::Input,
    ) -> Result<Option<VersionId>, Box<dyn std::error::Error + Send + Sync>> {
        let maybe_raw_version = input
            .get(&self.header_name)
            .and_then(|v| Some(v.to_str()))
            .transpose()?;

        let raw_version = if let Some(raw_version) = maybe_raw_version {
            raw_version
        } else {
            return Ok(None);
        };

        let version = VersionId::try_from(raw_version).map(Some).map_err(|e| {
            // TODO: Fix this error handling
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ))
        })?;

        Ok(version)
    }
}
