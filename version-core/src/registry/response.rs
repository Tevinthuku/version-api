use crate::version::{ErasedVersionChangeTransformer, Version, VersionId};
use itertools::Itertools;
use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct ApiResponseResourceRegistry {
    versions: HashMap<TypeId, ApiResourceVersionChanges>,
}

#[derive(Default)]
struct ApiResourceVersionChanges {
    data: HashMap<VersionId, Box<dyn ErasedVersionChangeTransformer>>,
}

impl ApiResponseResourceRegistry {
    pub fn transform(
        &self,
        response_body: impl std::any::Any,
        pinned_api_version: impl Into<VersionId>,
    ) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>> {
        let pinned_api_version = pinned_api_version.into();

        let resource_type_id = response_body.type_id();
        let mut response_body = Box::new(response_body) as Box<dyn std::any::Any>;
        if let Some(resource_version_changes) = self.versions.get(&resource_type_id) {
            let transformers = resource_version_changes
                .data
                .iter()
                // sorting in descending order, latest versions first
                .sorted_by(|a, b| b.0.cmp(&a.0))
                // apply transformations introduced above the pinned version boundary
                .take_while(|(version, _)| &pinned_api_version < *version);

            for (_, transformer) in transformers {
                response_body = transformer.transform(response_body)?;
            }
        }
        Ok(response_body)
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
