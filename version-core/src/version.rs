use bytes::Bytes;
use std::any::Any;
use std::any::TypeId;
use version_id::VersionId;

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Request,
    Response,
}

#[doc(hidden)]
// Internal type-erased adapter used by the registry.
// `VersionChangeTransformer` has associated types (`Input`/`Output`), so each
// implementation has a different concrete type and can't be stored directly
// in one heterogeneous collection. This trait erases those concrete types by
// accepting/returning `Bytes`, allowing us to keep all transformers in
// the same registry map and invoke them dynamically at runtime.
pub trait ErasedVersionChangeTransformer: Send + Sync {
    fn resource_type(&self) -> ResourceType;
    fn head_version(&self) -> TypeId;
    fn transform(&self, value: Bytes) -> Result<Bytes, Box<dyn std::error::Error>>;
}

pub struct Version {
    pub id: VersionId,
    pub changes: Vec<Box<dyn ErasedVersionChangeTransformer>>,
}

pub trait VersionChange {
    fn description() -> &'static str;
}

pub trait RequestChangeHistory {
    type Head: Any + 'static;
    fn version_ids() -> Vec<VersionId>;
    fn register(
        registry: &mut crate::registry::ResourceRegistry,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait VersionChangeTransformer {
    type Input: Any + 'static;
    type Output: Any + 'static;
    fn resource_type(&self) -> ResourceType;
    fn description(&self) -> &str;
    fn head_version(&self) -> TypeId;
    fn transform(&self, value: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>>;
}

impl<T: Send + Sync> ErasedVersionChangeTransformer for T
where
    T: VersionChangeTransformer + 'static,
    T::Input: serde::de::DeserializeOwned,
    T::Output: serde::Serialize,
{
    fn resource_type(&self) -> ResourceType {
        VersionChangeTransformer::resource_type(self)
    }
    fn head_version(&self) -> TypeId {
        VersionChangeTransformer::head_version(self)
    }

    fn transform(&self, value: Bytes) -> Result<Bytes, Box<dyn std::error::Error>> {
        let input: T::Input = serde_json::from_slice(&value)?;
        let output = VersionChangeTransformer::transform(self, input)?;
        let serialized = serde_json::to_vec(&output)?;
        Ok(Bytes::from(serialized))
    }
}
