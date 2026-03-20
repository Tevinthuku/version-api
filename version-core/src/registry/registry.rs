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

#[derive(Debug, Clone, Copy)]
pub enum TransformDirection {
    Request,
    Response,
}

#[derive(Debug, Clone)]
pub struct TransformContext {
    pub direction: TransformDirection,
    pub user_version: VersionId,
    pub head_type: TypeId,
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
        ctx: TransformContext,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let serialized = serde_json::to_vec(&data)?;
        let mut bytes = Bytes::from(serialized);

        let type_id = ctx.head_type;
        let maybe_resource_version_changes = match ctx.direction {
            TransformDirection::Response => self.response_versions.get(&type_id),
            TransformDirection::Request => self.request_versions.get(&type_id),
        };

        if let Some(resource_version_changes) = maybe_resource_version_changes {
            let transformers = resource_version_changes
                .data
                .iter()
                .filter(|(transformer_version, _)| &ctx.user_version < *transformer_version)
                .sorted_by(|a, b| match &ctx.direction {
                    TransformDirection::Request => a.0.cmp(b.0),
                    TransformDirection::Response => b.0.cmp(a.0),
                });

            for (_version, transformer) in transformers {
                bytes = transformer.transform(bytes)?;
            }
        }
        Ok(bytes)
    }
}
