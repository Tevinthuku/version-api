use actix_web::body::BoxBody;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpResponse, Responder, body::EitherBody, error::JsonPayloadError};
use actix_web::{mime, web};
use serde::Serialize;
use version_core::registry::ApiResponseResourceRegistry;
use version_id::VersionId;
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
        let body = serde_json::to_vec(&self.0)
            .map_err(|err| Box::new(JsonPayloadError::Serialize(err)))?;
        let mut transformed_body = body;

        if let Some(registry) = registry {
            let header_name = registry.header_name();
            let version = try_parse_version_from_header(req.headers(), header_name)?;

            if let Some(version) = version {
                transformed_body = registry.transform(self.0, version)?.to_vec();
            }
        }
        Ok(HttpResponse::Ok()
            .content_type(mime::APPLICATION_JSON)
            .body(transformed_body)
            .map_into_left_body())
    }
}

fn try_parse_version_from_header(
    headers: &HeaderMap,
    header_name: &str,
) -> Result<Option<VersionId>, Box<dyn std::error::Error>> {
    let version = headers.get(header_name);
    if let Some(version) = version {
        let version = version
            .to_str()
            // todo: Fix error handling, implement my own error type for usecases where I need a custom error message string but still need to preserve the full error chain
            .map_err(|err| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, err)))?;
        VersionId::try_from(version).map(Some)
    } else {
        Ok(None)
    }
}
