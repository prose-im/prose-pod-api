// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod redoc;

use axum::routing::get;

pub use self::redoc::*;

pub(super) fn router() -> axum::Router {
    axum::Router::new().route("/api-docs/redoc", get(redoc_route))
}
