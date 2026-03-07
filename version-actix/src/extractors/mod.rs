use version_id::{VersionId, VersionIdValidator};

pub struct BaseActixVersionIdExtractor {
    extractor_type: BaseActixVersionIdExtractorType,
    version_validator: Box<dyn VersionIdValidator>,
}

enum BaseActixVersionIdExtractorType {
    Header { header_name: String },
}

impl BaseActixVersionIdExtractorType {
    fn attribute_name(&self) -> &str {
        match self {
            BaseActixVersionIdExtractorType::Header { header_name } => header_name,
        }
    }
}

impl BaseActixVersionIdExtractor {
    pub fn header_extractor(
        header_name: String,
        version_validator: Box<dyn VersionIdValidator + 'static>,
    ) -> Box<dyn ActixVersionIdExtractor> {
        Box::new(Self {
            extractor_type: BaseActixVersionIdExtractorType::Header { header_name },
            version_validator,
        })
    }
}

pub trait ActixVersionIdExtractor: Send + Sync {
    fn extract(
        &self,
        req: &actix_web::HttpRequest,
    ) -> Result<Option<VersionId>, Box<dyn std::error::Error>>;
}

impl ActixVersionIdExtractor for BaseActixVersionIdExtractor {
    fn extract(
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
