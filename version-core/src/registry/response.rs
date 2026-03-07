use crate::version::{ErasedVersionChangeTransformer, Version};
use bytes::Bytes;
use itertools::Itertools;
use std::{any::TypeId, collections::HashMap};
use version_id::VersionId;

#[derive(Default)]
pub struct ApiResponseResourceRegistry {
    versions: HashMap<TypeId, ApiResourceVersionChanges>,
}

#[derive(Default)]
struct ApiResourceVersionChanges {
    data: HashMap<VersionId, Box<dyn ErasedVersionChangeTransformer>>,
}

impl ApiResponseResourceRegistry {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }
    pub fn transform(
        &self,
        response_body: impl std::any::Any + serde::Serialize,
        api_version: VersionId,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let resource_type_id = response_body.type_id();
        let serialized = serde_json::to_vec(&response_body)?;
        let mut bytes = Bytes::from(serialized);

        if let Some(resource_version_changes) = self.versions.get(&resource_type_id) {
            let transformers = resource_version_changes
                .data
                .iter()
                // sorting in descending order, latest versions first
                .sorted_by(|a, b| b.0.cmp(&a.0))
                // apply transformations introduced above the pinned version boundary
                .take_while(|(version, _)| &api_version < *version);

            for (_, transformer) in transformers {
                bytes = transformer.transform(bytes)?;
            }
        }
        Ok(bytes)
    }

    pub fn register(&mut self, version: Version) {
        let version_change = version.id;
        for change in version.changes {
            let head_version = change.head_version();
            self.versions
                .entry(head_version)
                .or_default()
                .data
                .insert(version_change.clone(), change);
        }
    }
}
