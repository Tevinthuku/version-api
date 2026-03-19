use crate::version::ErasedVersionChangeTransformer;
use crate::version::Version;
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
    versions: HashMap<TypeId, ApiResourceVersionChanges>,
}

#[derive(Debug, Clone)]
pub enum TransformDirection {
    UpForRequests { from: VersionId },
    DownForResponses { from: VersionId },
}

impl ResourceRegistry {
    pub fn register_version(&mut self, version: Version) {
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

    pub fn transform(
        &self,
        data: impl std::any::Any + serde::Serialize,
        direction: TransformDirection,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let resource_type_id = data.type_id();
        let serialized = serde_json::to_vec(&data)?;
        let mut bytes = Bytes::from(serialized);

        if let Some(resource_version_changes) = self.versions.get(&resource_type_id) {
            let transformers = resource_version_changes
                .data
                .iter()
                .sorted_by(|a, b| match &direction {
                    // sorting in ascending order, oldest versions first
                    // we want to apply the transformers from the oldest to the latest
                    TransformDirection::UpForRequests { from: _ } => a.0.cmp(b.0),
                    // sorting in descending order, latest versions first
                    // we want to apply the transformers from the latest to the oldest
                    TransformDirection::DownForResponses { from: _ } => b.0.cmp(a.0),
                })
                .take_while(|(version, _)| match &direction {
                    TransformDirection::UpForRequests { from } => from > *version,
                    TransformDirection::DownForResponses { from } => from < *version,
                });

            for (_, transformer) in transformers {
                bytes = transformer.transform(bytes)?;
            }
        }
        Ok(bytes)
    }
}
