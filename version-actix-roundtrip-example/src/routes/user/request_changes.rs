use serde::Deserialize;
use serde::Serialize;
use version_core::RequestChangeHistory;
use version_core::VersionChange;

use crate::routes::api_version::ApiVersion;
use crate::routes::user::CreateUserRequest;

#[derive(Debug, Serialize, Deserialize, VersionChange)]
#[description = "Clients before v2.0.0 send `full_name` instead of split fields"]
pub struct LegacyCreateUserRequestV1 {
    pub full_name: String,
}

#[derive(Debug, Serialize, Deserialize, VersionChange)]
#[description = "Clients before v1.0.0 send `name` instead of `full_name`"]
pub struct LegacyCreateUserRequestV0_5 {
    pub name: String,
}

impl From<LegacyCreateUserRequestV0_5> for LegacyCreateUserRequestV1 {
    fn from(request: LegacyCreateUserRequestV0_5) -> Self {
        Self { full_name: request.name }
    }
}

impl From<LegacyCreateUserRequestV1> for CreateUserRequest {
    fn from(request: LegacyCreateUserRequestV1) -> Self {
        let mut parts = request.full_name.splitn(2, ' ');
        Self {
            first_name: parts.next().unwrap_or_default().to_string(),
            last_name: parts.next().unwrap_or_default().to_string(),
        }
    }
}

#[derive(RequestChangeHistory)]
#[head(CreateUserRequest)]
#[changes(
    below(ApiVersion::V2_0_0) => LegacyCreateUserRequestV1,
    below(ApiVersion::V1_0_0) => LegacyCreateUserRequestV0_5,
)]
pub struct CreateUserRequestHistory;
