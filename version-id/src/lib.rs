use semver::{BuildMetadata, Prerelease};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionId(semver::Version);

impl<S: Into<String>> From<S> for VersionId {
    fn from(value: S) -> Self {
        let version = semver::Version {
            major: 1,
            minor: 0,
            patch: 0,
            // TODO: Introduce better error handling here
            pre: Prerelease::new(value.into().as_str()).unwrap(),
            build: BuildMetadata::EMPTY,
        };
        VersionId(version)
    }
}
