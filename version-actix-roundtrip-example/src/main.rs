mod routes;
use std::env;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::web;
use version_actix::ActixVersionIdExtractor;
use version_actix::BaseActixVersionIdExtractor;
use version_core::registry::ResourceRegistry;

use crate::routes::api_version::ApiVersion;
use crate::routes::user::create_user;
use crate::routes::user::request_changes::CreateUserRequestHistory;
use crate::routes::user::response_changes::CreateUserResponseHistory;

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

    cfg.app_data(web::Data::new(registry)).app_data(version_id_extractor).service(create_user);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    HttpServer::new(|| App::new().configure(build_app_config))
        .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
        .run()
        .await
}
