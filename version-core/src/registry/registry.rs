use crate::version::Version;
use crate::version::{ErasedVersionChangeTransformer, ResourceType};
use bytes::Bytes;
use itertools::Itertools;
use std::any::TypeId;
use std::collections::HashMap;
use version_id::VersionId;

#[derive(Default)]
struct ApiResourceVersionChanges {
    data: HashMap<VersionId, Box<dyn ErasedVersionChangeTransformer>>,
}

#[derive(Default)]
pub struct ResourceRegistry {
    request_versions: HashMap<TypeId, ApiResourceVersionChanges>,
    response_versions: HashMap<TypeId, ApiResourceVersionChanges>,
}

#[derive(Debug, Clone)]
pub enum TransformDirection {
    UpForRequests { user_version: VersionId },
    DownForResponses { user_version: VersionId },
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, version: Version) {
        let version_change = version.id;
        for change in version.changes {
            let head_version = change.head_version();
            match change.resource_type() {
                ResourceType::Request => {
                    self.request_versions
                        .entry(head_version)
                        .or_default()
                        .data
                        .insert(version_change.clone(), change);
                }
                ResourceType::Response => {
                    self.response_versions
                        .entry(head_version)
                        .or_default()
                        .data
                        .insert(version_change.clone(), change);
                }
            }
        }
    }

    pub fn transform(
        &self,
        data: impl std::any::Any + serde::Serialize,
        direction: TransformDirection,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let resource_type_id = data.type_id();
        let serialized = serde_json::to_vec(&data)?;
        let mut bytes = Bytes::from(serialized);

        let maybe_resource_version_changes = match direction {
            TransformDirection::DownForResponses { .. } => {
                self.response_versions.get(&resource_type_id)
            }
            TransformDirection::UpForRequests { .. } => {
                self.request_versions.get(&resource_type_id)
            }
        };

        if let Some(resource_version_changes) = maybe_resource_version_changes {
            let transformers = match &direction {
                TransformDirection::DownForResponses { user_version } => resource_version_changes
                    .data
                    .iter()
                    .filter(|(transformer_version, _)| user_version < *transformer_version)
                    // sorting in descending order, latest versions first
                    .sorted_by(|a, b| b.0.cmp(a.0)),
                TransformDirection::UpForRequests { user_version } => resource_version_changes
                    .data
                    .iter()
                    .filter(|(transformer_version, _)| user_version > *transformer_version)
                    // sorting in ascending order, oldest versions first
                    .sorted_by(|a, b| a.0.cmp(b.0)),
            };

            for (_, transformer) in transformers {
                bytes = transformer.transform(bytes)?;
            }
        }
        Ok(bytes)
    }
}
