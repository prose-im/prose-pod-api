// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, Json};
use service::app_config::{PodAddress, PodConfig};

use crate::AppState;

pub async fn get_pod_config_route(
    State(AppState { app_config, .. }): State<AppState>,
) -> Json<PodConfig> {
    Json(app_config.read().unwrap().pod.clone())
}

pub(super) async fn get_pod_address_route(
    State(AppState { app_config, .. }): State<AppState>,
) -> Json<PodAddress> {
    Json(app_config.read().unwrap().pod.clone().address)
}
