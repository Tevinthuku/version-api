use crate::responder::VersionedJsonResponder;
use actix_web::{Result, get, web};
use serde::Deserialize;
use serde::Serialize;
use version_core::{
    ApiVersionId, ChangeHistory, VersionChange, registry::ApiResponseResourceRegistry,
};
mod responder;

#[derive(Serialize, Deserialize)]
struct MyObj {
    first_name: String,
    last_name: String,
}

#[get("/a/{name}")]
async fn index(name: web::Path<String>) -> Result<VersionedJsonResponder<MyObj>> {
    let obj = MyObj {
        first_name: name.to_string(),
        last_name: name.to_string(),
    };
    Ok(VersionedJsonResponder(obj))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let mut registry = ApiResponseResourceRegistry::new("X-API-Version".to_string());
    MyObjResponseHistoryVersions::register(&mut registry).unwrap();
    let registry = web::Data::new(registry);
    HttpServer::new(move || App::new().service(index).app_data(registry.clone()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(ApiVersionId)]
pub enum MyApiVersions {
    #[version("2.0.0")]
    V2_0_0,
    #[version("1.0.0")]
    V1_0_0,
    #[version("0.9.0")]
    V0_9_0,
}
// build version change history for MyObj
#[derive(ChangeHistory)]
#[head(MyObj)]
#[changes(
    below(MyApiVersions::V2_0_0) => CollapseNamesToSingleOne,
)]
struct MyObjResponseHistoryVersions;

#[derive(VersionChange, Serialize, Deserialize)]
#[description = "Names will now render a single string instead of two separate strings"]
struct CollapseNamesToSingleOne {
    name: String,
}

impl From<MyObj> for CollapseNamesToSingleOne {
    fn from(obj: MyObj) -> Self {
        Self {
            name: format!("{} {}", obj.first_name, obj.last_name),
        }
    }
}
