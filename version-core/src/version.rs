use std::any::{Any, TypeId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionId(String);

impl From<&str> for VersionId {
    fn from(value: &str) -> Self {
        VersionId(value.to_string())
    }
}

#[doc(hidden)]
pub trait InternalVersionChangeSetTransformer {
    fn head_version(&self) -> TypeId;
    fn transform(
        &self,
        value: Box<dyn std::any::Any>,
    ) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>>;
}

pub struct Version {
    pub id: VersionId,
    pub changes: Vec<Box<dyn InternalVersionChangeSetTransformer>>,
}

pub trait VersionChange {
    fn below_version() -> VersionId;
    fn description() -> &'static str;
}

pub trait ChangeHistory {
    type Head: Any + 'static;
    fn version_ids() -> Vec<VersionId>;
    fn register(registry: &mut crate::registry::ApiResponseResourceRegistry);
}

// This trait is implemented by generated transformer types.
pub trait VersionChangeTransformer {
    type Input: Any + 'static;
    type Output: Any + 'static;

    fn description(&self) -> &str;
    fn head_version(&self) -> TypeId;
    fn transform(&self, value: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>>;
}

impl<T> InternalVersionChangeSetTransformer for T
where
    T: VersionChangeTransformer + 'static,
{
    fn head_version(&self) -> TypeId {
        VersionChangeTransformer::head_version(self)
    }

    fn transform(
        &self,
        value: Box<dyn std::any::Any>,
    ) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>> {
        let input = value
            .downcast::<T::Input>()
            // TODO: handle this error better in a separate PR:
            .map_err(|_| "Failed to downcast input value".to_string())?;
        let output = VersionChangeTransformer::transform(self, *input)?;
        Ok(Box::new(output))
    }
}
