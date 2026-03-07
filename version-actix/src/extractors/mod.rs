use version_id::{VersionId, VersionIdValidator};

pub struct ActixVersionIdExtractor {
    extractor_type: ActixVersionIdExtractorType,
    version_validator: Box<dyn VersionIdValidator>,
}

enum ActixVersionIdExtractorType {
    Header { header_name: String },
}

impl ActixVersionIdExtractorType {
    fn attribute_name(&self) -> &str {
        match self {
            ActixVersionIdExtractorType::Header { header_name } => header_name,
        }
    }
}

impl ActixVersionIdExtractor {
    pub fn header_extractor(
        header_name: String,
        version_validator: Box<dyn VersionIdValidator + 'static>,
    ) -> Self {
        Self {
            extractor_type: ActixVersionIdExtractorType::Header { header_name },
            version_validator,
        }
    }
}

impl ActixVersionIdExtractor {
    pub fn extract(
        &self,
        req: &actix_web::HttpRequest,
    ) -> Result<Option<VersionId>, Box<dyn std::error::Error>> {
        let headers = req.headers();
        let maybe_raw_version = headers
            .get(self.extractor_type.attribute_name())
            .and_then(|v| Some(v.to_str()))
            .transpose()?;

        let raw_version = if let Some(raw_version) = maybe_raw_version {
            raw_version
        } else {
            return Ok(None);
        };

        let version = self.version_validator.validate(raw_version).map_err(|e| {
            // TODO: Fix this error handling
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ))
        })?;

        Ok(Some(version))
    }
}
