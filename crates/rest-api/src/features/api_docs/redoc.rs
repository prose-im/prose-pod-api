// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::fs::NamedFile;

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

pub async fn redoc_route_axum() {
    todo!()
}
