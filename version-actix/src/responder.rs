use actix_web::body::BoxBody;
use actix_web::{HttpResponse, Responder, body::EitherBody, error::JsonPayloadError};
use actix_web::{mime, web};
use serde::Serialize;
use version_core::registry::ApiResponseResourceRegistry;

use crate::extractors::ActixVersionIdExtractor;

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
        let registry = req.app_data::<web::Data<ApiResponseResourceRegistry>>();
        let version_id_extractor = req.app_data::<web::Data<Box<dyn ActixVersionIdExtractor>>>();

        let body = serde_json::to_vec(&self.0)
            .map_err(|err| Box::new(JsonPayloadError::Serialize(err)))?;

        let mut transformed_body = body;

        if let Some((registry, version_extractor)) = registry.zip(version_id_extractor) {
            let version_id = version_extractor.extract(req)?;
            if let Some(version_id) = version_id {
                transformed_body = registry.transform(self.0, version_id)?.to_vec();
            }
        }
        Ok(HttpResponse::Ok()
            .content_type(mime::APPLICATION_JSON)
            .body(transformed_body)
            .map_into_left_body())
    }
}
