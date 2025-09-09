// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod extractors;

use axum::middleware::from_extractor_with_state;
use axum::routing::{get, MethodRouter};
use axum::Json;
use axum::{extract::State, response::NoContent};
use serdev::Serialize;
use service::licensing::licensing_controller::{self, GetLicenseResponse};
use service::members::MemberRepository;
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
        ref license_service,
        ..
    }): State<AppState>,
) -> Json<GetLicenseResponse> {
    let response = licensing_controller::get_license(license_service).await;
    Json(response)
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

#[derive(Serialize)]
struct GetLicensingStatusResponse {
    pub license: GetLicenseResponse,
    pub user_count: u64,
    pub remaining_seats: u64,
}

async fn get_licensing_status(
    State(AppState {
        ref license_service,
        ref db,
        ..
    }): State<AppState>,
) -> Result<Json<GetLicensingStatusResponse>, Error> {
    let license = licensing_controller::get_license(license_service).await;
    let user_count = MemberRepository::count(db).await?;
    let remaining_seats = (license.user_limit as u64)
        .checked_sub(user_count)
        .unwrap_or_default();

    Ok(Json(GetLicensingStatusResponse {
        license,
        user_count,
        remaining_seats,
    }))
}
