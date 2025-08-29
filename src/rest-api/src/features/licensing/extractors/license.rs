// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{body::Bytes, extract::FromRequest};
use axum_extra::headers::{ContentType, HeaderMapExt};

use crate::extractors::prelude::*;

impl FromRequest<AppState> for service::licensing::License {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::extract::license", level = "trace", skip_all)]
    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let ref license_service = state.license_service;

        let content_type = (req.headers())
            .typed_get::<ContentType>()
            // Assume binary data by default.
            .unwrap_or(ContentType::octet_stream());

        let body_bytes = match Bytes::from_request(req, state).await {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(Error::from(error::BadRequest {
                    reason: format!("Invalid body: {err}"),
                }))
            }
        };

        if content_type == ContentType::text() {
            let body_string = std::str::from_utf8(&body_bytes).map_err(|err| {
                Error::from(error::BadRequest {
                    reason: format!("Invalid base64 license: {err}"),
                })
            })?;
            match license_service.deserialize_license_base64(body_string) {
                Ok(license) => Ok(license),
                Err(err) => Err(Error::from(error::BadRequest {
                    reason: format!("Invalid base64 license: {err}"),
                })),
            }
        } else if content_type == ContentType::octet_stream() {
            match license_service.deserialize_license_bytes(&body_bytes) {
                Ok(license) => Ok(license),
                Err(err) => Err(Error::from(error::BadRequest {
                    reason: format!("Invalid raw license: {err}"),
                })),
            }
        } else {
            Err(Error::from(error::BadRequest {
                reason: format!("Invalid `Content-Type`: {content_type}"),
            }))
        }
    }
}
