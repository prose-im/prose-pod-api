// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{http::StatusCode, routing::*};
use tower_http::services::ServeDir;

pub(crate) fn router() -> axum::Router {
    axum::Router::new().nest_service(
        "/api-docs",
        get_service(ServeDir::new("static/api-docs")).handle_error(|error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {error}"),
            )
        }),
    )
}
