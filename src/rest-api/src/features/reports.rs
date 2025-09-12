// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::Json;
use iso8601_timestamp::Timestamp;
use serdev::Serialize;
use service::{
    app_config::DashboardConfig,
    licensing::licensing_controller::{self, GetLicensingStatusResponse},
    pod_version::{pod_version_controller, PodComponentsVersions},
    server_config::{server_config_controller, PublicServerConfig},
    workspace::{workspace_controller, Workspace, WorkspaceService},
    AppConfig,
};

use crate::{error::Error, AppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/reports/cloud-api", get(get_cloud_api_report))
        .with_state(app_state)
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct GetCloudApiReportResponse {
    pub timestamp: Timestamp,
    pub versions: PodComponentsVersions,
    pub licensing: GetLicensingStatusResponse,
    pub workspace: Workspace,
    pub server: PublicServerConfig,
    pub dashboard: Arc<DashboardConfig>,
}

async fn get_cloud_api_report(
    State(AppState {
        ref db,
        ref license_service,
        ref pod_version_service,
        ..
    }): State<AppState>,
    State(ref app_config): State<Arc<AppConfig>>,
    ref workspace_service: WorkspaceService,
) -> Result<Json<GetCloudApiReportResponse>, Error> {
    let timestamp = Timestamp::now_utc();
    let versions = pod_version_controller::get_pod_version(pod_version_service).await?;
    let licensing = licensing_controller::get_licensing_status(license_service, db).await?;
    let workspace = workspace_controller::get_workspace(workspace_service).await?;
    let server = server_config_controller::get_server_config_public(app_config);
    let dashboard = app_config.dashboard.clone();

    Ok(Json(GetCloudApiReportResponse {
        timestamp,
        versions,
        licensing,
        workspace,
        server,
        dashboard,
    }))
}
