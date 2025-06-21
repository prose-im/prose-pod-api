// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod routes;

use axum::middleware::from_extractor_with_state;
use axum::routing::MethodRouter;
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/pod/config",
            MethodRouter::new().get(get_pod_config_route),
        )
        .route(
            "/v1/pod/config/address",
            MethodRouter::new().get(get_pod_address_route),
        )
        .route(
            "/v1/pod/config/dashboard-url",
            MethodRouter::new().get(get_dashboard_url_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
