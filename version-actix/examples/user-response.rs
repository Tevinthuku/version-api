use actix_web::{Result, get, web};
use serde::Deserialize;
use serde::Serialize;
use version_actix::{ActixVersionIdExtractor, VersionedJsonResponder};
use version_core::{
    ApiVersionId, ChangeHistory, VersionChange, registry::ApiResponseResourceRegistry,
};

#[derive(Serialize, Deserialize)]
struct CurrentUser {
    first_name: String,
    last_name: String,
}

#[get("/a/{name}")]
async fn index(name: web::Path<String>) -> Result<VersionedJsonResponder<CurrentUser>> {
    let obj = CurrentUser {
        first_name: name.to_string(),
        last_name: name.to_string(),
    };
    Ok(VersionedJsonResponder(obj))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let mut registry = ApiResponseResourceRegistry::new();
    CurrentUserResponseHistoryVersions::register(&mut registry).unwrap();
    let version_id_extractor = ActixVersionIdExtractor::header_extractor(
        "X-API-Version".to_string(),
        ApiVersion::validator(),
    );

    let registry = web::Data::new(registry);
    let version_id_extractor = web::Data::new(version_id_extractor);
    HttpServer::new(move || {
        App::new()
            .service(index)
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

#[derive(ChangeHistory)]
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
