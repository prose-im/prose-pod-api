// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod redoc;

use axum::{http::StatusCode, routing::*};
use tower_http::services::ServeDir;

pub use self::redoc::*;

pub(crate) fn router() -> axum::Router {
    axum::Router::new()
        .route("/api-docs/redoc", get(redoc_route))
        .nest_service(
            "/api-docs",
            get_service(ServeDir::new("static/api-docs")).handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
}
