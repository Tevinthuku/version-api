use serde::Deserialize;
use serde::Serialize;
use version_core::ResponseChangeHistory;
use version_core::VersionChange;

use crate::routes::api_version::ApiVersion;
use crate::routes::user::CreateUserResponse;

#[derive(Debug, Serialize, Deserialize, VersionChange)]
#[description = "Clients before v2.0.0 expect `full_name` instead of split fields"]
pub struct LegacyCreateUserResponseV1 {
    pub full_name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, VersionChange)]
#[description = "Clients before v1.0.0 expect `name` and a boolean success flag"]
pub struct LegacyCreateUserResponseV0_5 {
    pub name: String,
    pub success: bool,
}

impl From<CreateUserResponse> for LegacyCreateUserResponseV1 {
    fn from(response: CreateUserResponse) -> Self {
        Self {
            full_name: format!("{} {}", response.first_name, response.last_name),
            status: response.status,
        }
    }
}

impl From<LegacyCreateUserResponseV1> for LegacyCreateUserResponseV0_5 {
    fn from(response: LegacyCreateUserResponseV1) -> Self {
        Self { name: response.full_name, success: response.status == "created" }
    }
}

#[derive(ResponseChangeHistory)]
#[head(CreateUserResponse)]
#[changes(
    below(ApiVersion::V2_0_0) => LegacyCreateUserResponseV1,
    below(ApiVersion::V1_0_0) => LegacyCreateUserResponseV0_5,
)]
pub struct CreateUserResponseHistory;
