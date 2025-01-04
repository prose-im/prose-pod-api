// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod pod_address;
mod pod_config;

use axum::middleware::from_extractor_with_state;
use axum::routing::get;
use axum::routing::MethodRouter;

use crate::AppState;

pub use self::pod_address::*;
pub use self::pod_config::*;

use super::auth::guards::IsAdmin;

pub const POD_ADDRESS_ROUTE: &'static str = "/v1/pod/config/address";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/pod/config", get(get_pod_config_route))
        .route(
            POD_ADDRESS_ROUTE,
            MethodRouter::new()
                .put(set_pod_address_route)
                .get(get_pod_address_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
