use actix_web::body::BoxBody;
use actix_web::{HttpResponse, Responder, body::EitherBody, error::JsonPayloadError};
use actix_web::{mime, web};
use serde::Serialize;
use version_core::registry::ApiResponseResourceRegistry;
use version_id::VersionId;
pub struct VersionedJsonResponder<T: Serialize + 'static>(pub T);

impl<T: Serialize + 'static> Responder for VersionedJsonResponder<T> {
    type Body = EitherBody<BoxBody>;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let registry = req.app_data::<web::Data<ApiResponseResourceRegistry>>();
        match serde_json::to_vec(&self.0) {
            Ok(body) => {
                let mut transformed_body = body;
                if let Some(registry) = registry {
                    let header_name = registry.header_name();
                    let version = req.headers().get(header_name).unwrap().to_str().unwrap();
                    let version = VersionId::try_from(version).unwrap();
                    match registry.transform(self.0, version) {
                        Ok(body) => {
                            transformed_body = body.to_vec();
                        }
                        Err(err) => {
                            return HttpResponse::from_error(err).map_into_right_body();
                        }
                    }
                }
                HttpResponse::Ok()
                    .content_type(mime::APPLICATION_JSON)
                    .body(transformed_body)
                    .map_into_left_body()
            }
            Err(err) => {
                HttpResponse::from_error(JsonPayloadError::Serialize(err)).map_into_right_body()
            }
        }
    }
}
