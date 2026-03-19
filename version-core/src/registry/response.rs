use crate::{
    registry::registry::{ResourceRegistry, TransformDirection},
    version::Version,
};
use bytes::Bytes;
use version_id::VersionId;

#[derive(Default)]
pub struct ApiResponseResourceRegistry {
    inner: ResourceRegistry,
}

impl ApiResponseResourceRegistry {
    pub fn new() -> Self {
        Self {
            inner: ResourceRegistry::default(),
        }
    }
    pub fn transform(
        &self,
        response_body: impl std::any::Any + serde::Serialize,
        api_version: VersionId,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        self.inner.transform(
            response_body,
            TransformDirection::DownForResponses {
                user_version: api_version,
            },
        )
    }

    pub fn register(&mut self, version: Version) {
        self.inner.register_version(version);
    }
}
