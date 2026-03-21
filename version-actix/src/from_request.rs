use std::any::TypeId;
use std::pin::Pin;

use actix_web::Error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::Payload;
use actix_web::web::Json;
use actix_web::web::{self};
use serde::Serialize;
use serde::de::DeserializeOwned;
use version_core::TransformDirection;
use version_core::registry::ResourceRegistry;
use version_core::registry::TransformContext;

use crate::ActixVersionIdExtractor;

pub struct VersionedJsonRequest<T: DeserializeOwned + Serialize + 'static>(pub T);

impl<T: DeserializeOwned + Serialize + 'static> FromRequest for VersionedJsonRequest<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = payload.take();
        Box::pin(async move {
            let registry = req.app_data::<web::Data<ResourceRegistry>>();
            let version_id_extractor = req.app_data::<web::Data<dyn ActixVersionIdExtractor>>();
            let json_body = Json::<serde_json::Value>::from_request(&req, &mut payload).await?;

            if let Some((registry, version_extractor)) = registry.zip(version_id_extractor) {
                let version_id = version_extractor.extract(&req)?;
                if let Some(version_id) = version_id {
                    let transformed_body = registry.transform(
                        json_body.0,
                        TransformContext {
                            direction: TransformDirection::Request,
                            head_type: TypeId::of::<T>(),
                            user_version: version_id,
                        },
                    )?;

                    let transformed_body = serde_json::from_slice::<T>(&transformed_body)?;

                    return Ok(VersionedJsonRequest(transformed_body));
                }
            }

            let json_body = serde_json::from_value::<T>(json_body.0)?;
            Ok(VersionedJsonRequest(json_body))
        })
    }
}

impl<T: DeserializeOwned + Serialize + 'static> VersionedJsonRequest<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
