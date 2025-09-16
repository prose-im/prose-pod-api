// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{http::StatusCode, routing::get};

pub(super) fn router() -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/healthz", get(health_check))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
