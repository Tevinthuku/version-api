mod versioning;

use actix_web::{App, HttpServer, Result, post, web};
use version_actix::{
    ActixVersionIdExtractor, BaseActixVersionIdExtractor, VersionedJsonRequest,
    VersionedJsonResponder,
};
use version_core::registry::ResourceRegistry;

use crate::versioning::api_version::ApiVersion;
use crate::versioning::request::{CreateUserRequest, CreateUserRequestHistory};
use crate::versioning::response::{CreateUserResponse, CreateUserResponseHistory};

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

fn build_app_config(cfg: &mut web::ServiceConfig) {
    let mut registry = ResourceRegistry::new();
    CreateUserRequestHistory::register(&mut registry).unwrap();
    CreateUserResponseHistory::register(&mut registry).unwrap();

    let version_id_extractor = BaseActixVersionIdExtractor::header_extractor(
        "X-API-Version".to_string(),
        ApiVersion::validator(),
    );
    let version_id_extractor: web::Data<dyn ActixVersionIdExtractor> =
        web::Data::from(version_id_extractor);

    cfg.app_data(web::Data::new(registry))
        .app_data(version_id_extractor)
        .service(create_user);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(build_app_config))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
