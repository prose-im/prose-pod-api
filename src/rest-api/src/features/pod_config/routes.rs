// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{extract::State, Json};
use service::{
    app_config::{PodAddress, PodConfig},
    AppConfig,
};

pub async fn get_pod_config_route(
    State(ref app_config): State<Arc<AppConfig>>,
) -> Json<Arc<PodConfig>> {
    Json(app_config.pod.clone())
}

pub(super) async fn get_pod_address_route(
    State(ref app_config): State<Arc<AppConfig>>,
) -> Json<PodAddress> {
    Json(app_config.pod.address.clone())
}
