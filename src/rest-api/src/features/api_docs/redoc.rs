// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
