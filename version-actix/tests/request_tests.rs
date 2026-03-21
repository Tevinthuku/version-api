use actix_web::App;
use actix_web::Result;
use actix_web::post;
use actix_web::test;
use actix_web::web;
use serde::Deserialize;
use serde::Serialize;
use version_actix::ActixVersionIdExtractor;
use version_actix::BaseActixVersionIdExtractor;
use version_actix::VersionedJsonRequest;
use version_core::ApiVersionId;
use version_core::RequestChangeHistory;
use version_core::VersionChange;
use version_core::registry::ResourceRegistry;

#[derive(Debug, Serialize, Deserialize, PartialEq, VersionChange)]
#[description = "The latest user request model, with the first and last name"]
struct CreateUserRequest {
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

#[derive(RequestChangeHistory)]
#[head(CreateUserRequest)]
#[changes(
    below(ApiVersion::V2_0_0) => LegacyCreateUserRequest,
)]
struct CreateUserRequestHistory;

#[derive(VersionChange, Serialize, Deserialize)]
#[description = "Users before v2.0.0 sent a single name field"]
struct LegacyCreateUserRequest {
    name: String,
}

impl From<LegacyCreateUserRequest> for CreateUserRequest {
    fn from(obj: LegacyCreateUserRequest) -> Self {
        // in this example scenario, we expect users were providing the first and last name as a single string, split by whiteSpace
        let mut parts = obj.name.splitn(2, ' ');
        Self {
            first_name: parts.next().unwrap_or_default().to_string(),
            last_name: parts.next().unwrap_or_default().to_string(),
        }
    }
}

#[post("/users")]
async fn create_user_endpoint(
    user: VersionedJsonRequest<CreateUserRequest>,
) -> Result<web::Json<CreateUserRequest>> {
    Ok(web::Json(user.into_inner()))
}

fn build_app_config(cfg: &mut web::ServiceConfig) {
    let mut registry = ResourceRegistry::default();
    CreateUserRequestHistory::register(&mut registry).unwrap();
    let version_id_extractor = BaseActixVersionIdExtractor::header_extractor(
        "X-API-Version".to_string(),
        ApiVersion::validator(),
    );
    let version_id_extractor: web::Data<dyn ActixVersionIdExtractor> =
        web::Data::from(version_id_extractor);

    cfg.app_data(web::Data::new(registry))
        .app_data(version_id_extractor)
        .service(create_user_endpoint);
}

#[actix_rt::test]
async fn v2_request_passes_through_unchanged() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("X-API-Version", "2.0.0"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"first_name": "Alice", "last_name": "Doe"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CreateUserRequest = test::read_body_json(resp).await;
    assert_eq!(body.first_name, "Alice");
    assert_eq!(body.last_name, "Doe");
}

#[actix_rt::test]
async fn pre_v2_request_upgrades_single_name_to_split_fields() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("X-API-Version", "1.0.0"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"name": "Alice Doe"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CreateUserRequest = test::read_body_json(resp).await;
    assert_eq!(body.first_name, "Alice");
    assert_eq!(body.last_name, "Doe");
}

#[actix_rt::test]
async fn no_version_header_passes_through_unchanged() {
    let app = test::init_service(App::new().configure(build_app_config)).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"first_name": "Alice", "last_name": "Doe"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CreateUserRequest = test::read_body_json(resp).await;
    assert_eq!(body.first_name, "Alice");
    assert_eq!(body.last_name, "Doe");
}
