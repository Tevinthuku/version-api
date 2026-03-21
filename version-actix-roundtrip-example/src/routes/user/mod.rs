use actix_web::Result;
use actix_web::post;
use serde::Deserialize;
use serde::Serialize;
use version_actix::VersionedJsonRequest;
use version_actix::VersionedJsonResponder;
use version_core::VersionChange;

pub mod request_changes;
pub mod response_changes;

#[derive(Debug, Serialize, Deserialize, VersionChange)]
#[description = "The latest request model expects first and last names separately"]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: String,
}

#[post("/users")]
async fn create_user(
    user: VersionedJsonRequest<CreateUserRequest>,
) -> Result<VersionedJsonResponder<CreateUserResponse>> {
    let user = user.into_inner();
    Ok(VersionedJsonResponder(CreateUserResponse {
        first_name: user.first_name,
        last_name: user.last_name,
        status: "created".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub first_name: String,
    pub last_name: String,
    pub status: String,
}
