use crate::version::{InternalVersionChangeSetTransformer, Version, VersionId};
use itertools::Itertools;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub(crate) struct ApiResponseResourceRegistry {
    versions: HashMap<TypeId, ApiResourceVersionChanges>,
}

#[derive(Default)]
struct ApiResourceVersionChanges {
    data: HashMap<VersionId, Box<dyn InternalVersionChangeSetTransformer>>,
}

impl ApiResponseResourceRegistry {
    pub(crate) fn transform(
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
                .take_while_inclusive(|(version, _)| version <= &&pinned_api_version);
            for (version, transformer) in transformers {
                println!("version: {:?}", version);

                response_body = transformer.transform(response_body)?;

                let response_body_type_id = response_body.type_id();
                println!("response_body_type_id: {:?}", response_body_type_id);
            }
        }
        Ok(response_body)
    }

    pub(crate) fn register(&mut self, version: Version) {
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
