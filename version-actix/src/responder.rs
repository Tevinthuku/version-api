use actix_web::body::BoxBody;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpResponse, Responder, body::EitherBody, error::JsonPayloadError};
use actix_web::{mime, web};
use serde::Serialize;
use version_core::registry::ApiResponseResourceRegistry;

pub struct VersionedJsonResponder<T: Serialize + 'static>(pub T);

impl<T: Serialize + 'static> Responder for VersionedJsonResponder<T> {
    type Body = EitherBody<BoxBody>;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match self.respond_to_inner(req) {
            Ok(response) => response,
            Err(err) => HttpResponse::from_error(err).map_into_right_body(),
        }
    }
}

impl<T: Serialize + 'static> VersionedJsonResponder<T> {
    fn respond_to_inner(
        self,
        req: &actix_web::HttpRequest,
    ) -> Result<HttpResponse<EitherBody<BoxBody>>, Box<dyn std::error::Error>> {
        let registry = req.app_data::<web::Data<ApiResponseResourceRegistry<HeaderMap>>>();
        let body = serde_json::to_vec(&self.0)
            .map_err(|err| Box::new(JsonPayloadError::Serialize(err)))?;
        let mut transformed_body = body;

        if let Some(registry) = registry {
            transformed_body = registry.transform(self.0, req.headers())?.to_vec();
        }
        Ok(HttpResponse::Ok()
            .content_type(mime::APPLICATION_JSON)
            .body(transformed_body)
            .map_into_left_body())
    }
}
