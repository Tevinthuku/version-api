use actix_web::{App, Result, get, test, web};
use serde::{Deserialize, Serialize};
use version_actix::VersionedJsonResponder;
use version_core::{
    ApiVersionId, ChangeHistory, VersionChange, registry::ApiResponseResourceRegistry,
};

use version_actix::VersionIdHeaderExtractor;

#[derive(Serialize, Deserialize)]
struct CurrentUser {
    first_name: String,
    last_name: String,
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

#[get("/user/{name}")]
async fn user_endpoint(name: web::Path<String>) -> Result<VersionedJsonResponder<CurrentUser>> {
    Ok(VersionedJsonResponder(CurrentUser {
        first_name: name.to_string(),
        last_name: "Doe".to_string(),
    }))
}

fn build_app_config(cfg: &mut web::ServiceConfig) {
    let mut registry = ApiResponseResourceRegistry::new(VersionIdHeaderExtractor::new(
        "X-API-Version".to_string(),
    ));
    CurrentUserResponseHistoryVersions::register(&mut registry).unwrap();
    cfg.app_data(web::Data::new(registry))
        .service(user_endpoint);
}

#[actix_rt::test]
async fn v2_header_returns_split_name_fields() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::get()
        .uri("/user/Alice")
        .insert_header(("X-API-Version", "2.0.0"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CurrentUser = test::read_body_json(resp).await;
    assert_eq!(body.first_name, "Alice");
    assert_eq!(body.last_name, "Doe");
}

#[actix_rt::test]
async fn pre_v2_header_returns_collapsed_name() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::get()
        .uri("/user/Alice")
        .insert_header(("X-API-Version", "1.0.0"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: UserWithSingleNameField = test::read_body_json(resp).await;
    assert_eq!(body.name, "Alice Doe");
}

#[actix_rt::test]
async fn no_version_header_returns_latest_format() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::get().uri("/user/Alice").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CurrentUser = test::read_body_json(resp).await;
    assert_eq!(body.first_name, "Alice");
    assert_eq!(body.last_name, "Doe");
}
