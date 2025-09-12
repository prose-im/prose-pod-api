// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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

mod redoc {
    use axum::{body::Body, http::Request, response::IntoResponse};
    use tower::Service as _;
    use tower_http::services::ServeFile;

    use crate::error::{self, Error};

    pub async fn redoc_route() -> Result<impl IntoResponse, Error> {
        ServeFile::new("static/api-docs/redoc.html")
            .call(Request::new(Body::empty()))
            .await
            .map_err(|e| {
                Error::from(error::NotFound {
                    reason: format!("{e}"),
                })
            })
    }
}
