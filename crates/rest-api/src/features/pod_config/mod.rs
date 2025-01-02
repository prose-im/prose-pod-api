// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod pod_address;
mod pod_config;

use axum::routing::{get, put};

pub use self::pod_address::*;
pub use self::pod_config::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        get_pod_config_route,
        set_pod_address_route,
        get_pod_address_route,
    ]
}

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/v1/pod/config", get(get_pod_config_route_axum))
        .route("/v1/pod/config/address", put(set_pod_address_route_axum))
        .route("/v1/pod/config/address", get(get_pod_address_route_axum))
}
