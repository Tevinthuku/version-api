use semver::{BuildMetadata, Prerelease};
use std::io::{Error, ErrorKind};

/// Wraps `semver::Version` so that version strings like "2024-01-15" are
/// compared correctly (e.g. "10" > "2") instead of lexicographically, while
/// keeping the public API limited to plain strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionId(semver::Version);

impl TryFrom<&str> for VersionId {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let version = semver::Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::new(value).map_err(|_| {
                Box::new(Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected character in the version string",
                ))
            })?,
        };
        Ok(VersionId(version))
    }
}

impl TryFrom<String> for VersionId {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
