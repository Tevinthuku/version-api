use bytes::Bytes;
use std::any::{Any, TypeId};
use version_id::VersionId;

#[doc(hidden)]
// Internal type-erased adapter used by the registry.
// `VersionChangeTransformer` has associated types (`Input`/`Output`), so each
// implementation has a different concrete type and can't be stored directly
// in one heterogeneous collection. This trait erases those concrete types by
// accepting/returning `Bytes`, allowing us to keep all transformers in
// the same registry map and invoke them dynamically at runtime.
pub trait ErasedVersionChangeTransformer {
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

pub trait ChangeHistory {
    type Head: Any + 'static;
    fn version_ids() -> Vec<VersionId>;
    fn register(
        registry: &mut crate::registry::ApiResponseResourceRegistry,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait VersionChangeTransformer {
    type Input: Any + 'static;
    type Output: Any + 'static;

    fn description(&self) -> &str;
    fn head_version(&self) -> TypeId;
    fn transform(&self, value: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>>;
}

impl<T> ErasedVersionChangeTransformer for T
where
    T: VersionChangeTransformer + 'static,
    T::Input: serde::de::DeserializeOwned,
    T::Output: serde::Serialize,
{
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
