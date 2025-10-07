// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod extractors;

use axum::middleware::from_extractor_with_state;
use axum::routing::{get, MethodRouter};
use axum::Json;
use axum::{extract::State, response::NoContent};
use service::licensing::licensing_controller::{
    self, GetLicenseResponse, GetLicensingStatusResponse,
};
use service::{auth::IsAdmin, licensing::License};

use crate::{error::Error, AppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/licensing/license",
            MethodRouter::new().get(get_license).put(set_license),
        )
        .route("/v1/licensing/status", get(get_licensing_status))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

async fn get_license(
    State(AppState {
        ref licensing_service,
        ..
    }): State<AppState>,
) -> Json<GetLicenseResponse> {
    let response = licensing_controller::get_license(licensing_service).await;
    Json(response)
}

async fn set_license(
    State(AppState {
        ref licensing_service,
        ..
    }): State<AppState>,
    license: License,
) -> Result<NoContent, Error> {
    match licensing_service.install_license(license) {
        Ok(()) => Ok(NoContent),
        Err(err) => Err(Error::from(err)),
    }
}

async fn get_licensing_status(
    State(AppState {
        ref licensing_service,
        ref user_repository,
        ..
    }): State<AppState>,
) -> Result<Json<GetLicensingStatusResponse>, Error> {
    match licensing_controller::get_licensing_status(licensing_service, user_repository).await {
        Ok(status) => Ok(Json(status)),
        Err(err) => Err(Error::from(err)),
    }
}

mod errors {
    use crate::error::prelude::*;

    impl HttpApiError for service::licensing::errors::UserLimitReached {
        fn code(&self) -> ErrorCode {
            ErrorCode {
                value: "user_limit_reached",
                http_status: StatusCode::FORBIDDEN,
                log_level: LogLevel::Error,
            }
        }
    }
}
