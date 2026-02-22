use std::any::{Any, TypeId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionId(String);

impl From<&str> for VersionId {
    fn from(value: &str) -> Self {
        VersionId(value.to_string())
    }
}

pub(crate) trait InternalVersionChangeSetTransformer {
    fn head_version(&self) -> TypeId;
    fn transform(
        &self,
        value: Box<dyn std::any::Any>,
    ) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>>;
}

pub(crate) struct Version {
    pub(crate) id: VersionId,
    pub(crate) changes: Vec<Box<dyn InternalVersionChangeSetTransformer>>,
}

// This trait is what users of the library will implement to define their version changesets
pub trait VersionChangeSetTransformer {
    type Input: Any + 'static;
    type Output: Any + 'static;

    fn description(&self) -> &str;
    fn head_version(&self) -> TypeId;
    fn transform(&self, value: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>>;
}

impl<T> InternalVersionChangeSetTransformer for T
where
    T: VersionChangeSetTransformer + 'static,
{
    fn head_version(&self) -> TypeId {
        VersionChangeSetTransformer::head_version(self)
    }

    fn transform(
        &self,
        value: Box<dyn std::any::Any>,
    ) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>> {
        let input = value
            .downcast::<T::Input>()
            // TODO: handle this error better in a separate PR:
            
            .map_err(|_| "Failed to downcast input value".to_string())?;
        let output = VersionChangeSetTransformer::transform(self, *input)?;
        Ok(Box::new(output))
    }
}
