// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod extractors;

use axum::middleware::from_extractor_with_state;
use axum::routing::MethodRouter;
use axum::Json;
use axum::{extract::State, response::NoContent};
use base64::Engine;
use serdev::Serialize;
use service::{auth::IsAdmin, licensing::License};

use crate::{error::Error, AppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/licensing/license",
            MethodRouter::new().get(get_license).put(set_license),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct GetLicenseResponse {
    pub id: String,
    pub name: String,
    pub user_limit: u32,
    pub expiry: Option<iso8601_timestamp::Timestamp>,
    pub ttl_ms: Option<u128>,
}

async fn get_license(
    State(AppState {
        ref license_service,
        ..
    }): State<AppState>,
) -> Result<Json<GetLicenseResponse>, Error> {
    use std::time::SystemTime;

    use base64::engine::general_purpose::STANDARD_NO_PAD as base64;
    use iso8601_timestamp::Timestamp as IsoTimestamp;

    let license = (license_service.installed_licenses().last())
        .expect("The Community license should always be installed")
        .to_owned();

    let expiry = license.expiry();

    Ok(Json(GetLicenseResponse {
        id: base64.encode(license.id()),
        name: license.name().to_owned(),
        user_limit: license.user_limit(),
        expiry: expiry.map(IsoTimestamp::from),
        ttl_ms: expiry.map(|t| {
            t.duration_since(SystemTime::now())
                .unwrap_or_default()
                .as_millis()
        }),
    }))
}

async fn set_license(
    State(AppState {
        ref license_service,
        ..
    }): State<AppState>,
    license: License,
) -> Result<NoContent, Error> {
    match license_service.install_license(license) {
        Ok(()) => Ok(NoContent),
        Err(err) => Err(Error::from(err)),
    }
}
