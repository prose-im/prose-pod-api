// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_dns_records;

use axum::{middleware::from_extractor_with_state, routing::get};
use service::auth::IsAdmin;

use crate::AppState;

pub use self::get_dns_records::*;

use super::NETWORK_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            NETWORK_ROUTE,
            axum::Router::new().route("/dns/records", get(get_dns_records_route)),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
