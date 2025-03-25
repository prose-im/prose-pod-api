// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, Json};
use service::pod_config::{PodConfig, PodConfigRepository};

use crate::{error::Error, AppState};

pub async fn get_pod_config_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Json<PodConfig>, Error> {
    let model = PodConfigRepository::get(&db).await?;
    let res = model.map(PodConfig::from).unwrap_or_default();
    Ok(Json(res))
}
