use crate::version::{ErasedVersionChangeTransformer, Version};
use bytes::Bytes;
use itertools::Itertools;
use std::{any::TypeId, collections::HashMap};
use version_id::{VersionId, VersionIdExtractor};

pub struct ApiResponseResourceRegistry<T: ?Sized + 'static> {
    versions: HashMap<TypeId, ApiResourceVersionChanges>,
    version_extractor: Box<dyn VersionIdExtractor<Input = T>>,
}

#[derive(Default)]
struct ApiResourceVersionChanges {
    data: HashMap<VersionId, Box<dyn ErasedVersionChangeTransformer>>,
}

impl<T> ApiResponseResourceRegistry<T> {
    pub fn new(version_extractor: impl VersionIdExtractor<Input = T> + 'static) -> Self {
        Self {
            versions: HashMap::new(),
            version_extractor: Box::new(version_extractor),
        }
    }
    pub fn transform(
        &self,
        response_body: impl std::any::Any + serde::Serialize,
        input: &T,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let resource_type_id = response_body.type_id();
        let serialized = serde_json::to_vec(&response_body)?;
        let mut bytes = Bytes::from(serialized);

        let maybe_version = self
            .version_extractor
            .extract(input)
            //TODO: implement proper error handling here, with my own domain type, the error should read, failed to extract valid version-id from input
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

        let api_version = if let Some(version) = maybe_version {
            version
        } else {
            // if we don't recognize the version-id, we simply don't transform the response body
            // ideally, we should log a warning here or something
            return Ok(bytes);
        };

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
