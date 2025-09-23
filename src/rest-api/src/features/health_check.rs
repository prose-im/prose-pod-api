// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::StatusCode, routing::get};

use crate::{AppState, MinimalAppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/healthz", get(health_check))
        .with_state(app_state)
}

pub fn minimal_router(app_state: MinimalAppState) -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/healthz", get(health_check))
        .with_state(app_state)
}

async fn health_check(State(app_state): State<MinimalAppState>) -> StatusCode {
    if app_state.lifecycle_manager.is_restarting() {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    }
}
