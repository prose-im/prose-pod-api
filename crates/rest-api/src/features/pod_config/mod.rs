// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod pod_address;
mod pod_config;

use axum::routing::get;
use axum::routing::MethodRouter;

pub use self::pod_address::*;
pub use self::pod_config::*;

pub const POD_ADDRESS_ROUTE: &'static str = "/v1/pod/config/address";

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/v1/pod/config", get(get_pod_config_route))
        .route(
            POD_ADDRESS_ROUTE,
            MethodRouter::new()
                .put(set_pod_address_route)
                .get(get_pod_address_route),
        )
}
