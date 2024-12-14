// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod redoc;

use axum::routing::get;

pub use self::redoc::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![redoc_route]
}

pub(super) fn router<S: Clone + std::marker::Send + std::marker::Sync + 'static>() -> axum::Router<S>
{
    axum::Router::new().route("/api-docs/redoc", get(redoc_route_axum))
}
