use std::any::TypeId;

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::body::BoxBody;
use actix_web::body::EitherBody;
use actix_web::error::JsonPayloadError;
use actix_web::mime;
use actix_web::web;
use serde::Serialize;
use version_core::TransformDirection;
use version_core::registry::ResourceRegistry;
use version_core::registry::TransformContext;

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
        let registry = req.app_data::<web::Data<ResourceRegistry>>();
        let version_id_extractor = req.app_data::<web::Data<dyn ActixVersionIdExtractor>>();

        let body = serde_json::to_vec(&self.0)
            .map_err(|err| Box::new(JsonPayloadError::Serialize(err)))?;

        let mut transformed_body = body;

        if let Some((registry, version_extractor)) = registry.zip(version_id_extractor) {
            let version_id = version_extractor.extract(req)?;
            if let Some(version_id) = version_id {
                transformed_body = registry
                    .transform(
                        self.0,
                        TransformContext {
                            direction: TransformDirection::Response,
                            head_type: TypeId::of::<T>(),
                            user_version: version_id,
                        },
                    )?
                    .to_vec();
            }
        }
        Ok(HttpResponse::Ok()
            .content_type(mime::APPLICATION_JSON)
            .body(transformed_body)
            .map_into_left_body())
    }
}
