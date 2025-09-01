// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, routing::get, Json};
use service::pod_version::{pod_version_controller, PodComponentsVersions, VersionInfo};

use crate::{error::Error, AppState, MinimalAppState};

pub(crate) fn minimal_router(app_state: MinimalAppState) -> axum::Router {
    axum::Router::new()
        .route("/version", get(get_api_version_route))
        .route("/v1/version", get(get_api_version_route))
        .with_state(app_state)
}

pub(crate) fn router(app_state: AppState) -> axum::Router {
    let router = axum::Router::new()
        .route("/version", get(get_api_version_route))
        .route("/pod/version", get(get_pod_version_route))
        .route("/server/version", get(get_server_version_route));

    (router.clone()).nest("/v1", router).with_state(app_state)
}

async fn get_api_version_route(
    State(MinimalAppState {
        ref static_pod_version_service,
        ..
    }): State<MinimalAppState>,
) -> Json<VersionInfo> {
    Json(pod_version_controller::get_api_version(
        static_pod_version_service,
    ))
}

async fn get_pod_version_route(
    State(AppState {
        ref pod_version_service,
        ..
    }): State<AppState>,
) -> Result<Json<PodComponentsVersions>, Error> {
    match pod_version_controller::get_pod_version(pod_version_service).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(Error::from(err)),
    }
}

async fn get_server_version_route(
    State(AppState {
        ref pod_version_service,
        ..
    }): State<AppState>,
) -> Result<Json<VersionInfo>, Error> {
    match pod_version_controller::get_server_version(pod_version_service).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(Error::from(err)),
    }
}
