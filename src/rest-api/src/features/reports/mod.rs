// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::State;
use axum::routing::get;
use axum::Json;
use iso8601_timestamp::Timestamp;
use serdev::Serialize;
use service::{
    licensing::licensing_controller::{self, GetLicenseResponse},
    members::MemberRepository,
    pod_version::{pod_version_controller, PodComponentsVersions},
};

use crate::{error::Error, AppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/reports/cloud-api", get(get_cloud_api_report))
        .with_state(app_state)
}

#[derive(Serialize)]
pub struct GetCloudApiReportResponse {
    pub timestamp: Timestamp,
    pub version: PodComponentsVersions,
    pub license: GetLicenseResponse,
    pub user_count: u64,
    pub remaining_seats: u64,
}

async fn get_cloud_api_report(
    State(AppState {
        ref license_service,
        ref pod_version_service,
        ref db,
        ..
    }): State<AppState>,
) -> Result<Json<GetCloudApiReportResponse>, Error> {
    let timestamp = Timestamp::now_utc();
    let version = pod_version_controller::get_pod_version(pod_version_service).await?;
    let license = licensing_controller::get_license(license_service).await;
    let user_count = MemberRepository::count(db).await?;
    let remaining_seats = (license.user_limit as u64)
        .checked_sub(user_count)
        .unwrap_or_default();

    Ok(Json(GetCloudApiReportResponse {
        timestamp,
        version,
        license,
        user_count,
        remaining_seats,
    }))
}
