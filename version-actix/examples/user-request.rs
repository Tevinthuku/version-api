use actix_web::{Result, get, post, web};
use serde::Deserialize;
use serde::Serialize;
use version_actix::{BaseActixVersionIdExtractor, VersionedJsonRequest, VersionedJsonResponder};
use version_core::{
    ApiVersionId, RequestChangeHistory, VersionChange,
    registry::ResourceRegistry,
};

#[derive(Serialize, Deserialize, VersionChange)]
#[description = "The latest user request model, with the first and last name"]
struct CurrentUser {
    first_name: String,
    last_name: String,
}

#[post("/users")]
async fn create_user(
    user: VersionedJsonRequest<CurrentUser>,
) -> Result<VersionedJsonResponder<CurrentUser>> {
    let user = user.into_inner();
    let obj = CurrentUser {
        first_name: user.first_name,
        last_name: user.last_name,
    };
    Ok(VersionedJsonResponder(obj))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let mut registry = ResourceRegistry::new();
    CurrentUserResponseHistoryVersions::register(&mut registry).unwrap();

    let version_id_extractor = web::Data::from(BaseActixVersionIdExtractor::header_extractor(
        "X-API-Version".to_string(),
        ApiVersion::validator(),
    ));
    let registry = web::Data::new(registry);
    HttpServer::new(move || {
        App::new()
            .service(create_user)
            .app_data(registry.clone())
            .app_data(version_id_extractor.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[derive(ApiVersionId)]
pub enum ApiVersion {
    #[version("2.0.0")]
    V2_0_0,
    #[version("1.0.0")]
    V1_0_0,
    #[version("0.9.0")]
    V0_9_0,
}

#[derive(RequestChangeHistory)]
#[head(CurrentUser)]
#[changes(
    below(ApiVersion::V2_0_0) => UserWithSingleNameField,
)]
struct CurrentUserResponseHistoryVersions;

#[derive(VersionChange, Serialize, Deserialize)]
#[description = "Users on less than v2.0.0 expect a single string for the name"]
struct UserWithSingleNameField {
    name: String,
}

impl From<CurrentUser> for UserWithSingleNameField {
    fn from(obj: CurrentUser) -> Self {
        Self {
            name: format!("{} {}", obj.first_name, obj.last_name),
        }
    }
}

impl From<UserWithSingleNameField> for CurrentUser {
    fn from(obj: UserWithSingleNameField) -> Self {
        Self {
            first_name: obj.name,
            last_name: "".to_string(),
        }
    }
}
