// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{body::Body, http::Request, response::IntoResponse};
use rocket::{fs::NamedFile, http::hyper::service::Service};
use tower_http::services::ServeFile;

use crate::error::{self, Error};

#[rocket::get("/api-docs/redoc")]
pub async fn redoc_route() -> Result<NamedFile, Error> {
    NamedFile::open("static/api-docs/redoc.html")
        .await
        .map_err(|e| {
            error::NotFound {
                reason: format!("{e}"),
            }
            .into()
        })
}

pub async fn redoc_route_axum() -> Result<impl IntoResponse, Error> {
    ServeFile::new("static/api-docs/redoc.html")
        .call(Request::new(Body::empty()))
        .await
        .map_err(|e| {
            Error::from(error::NotFound {
                reason: format!("{e}"),
            })
        })
}
