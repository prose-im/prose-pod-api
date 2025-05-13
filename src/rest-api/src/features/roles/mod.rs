// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_role;

use axum::{middleware::from_extractor_with_state, routing::put};
use service::auth::IsAdmin;

use crate::AppState;

pub use self::set_member_role::*;

use super::members::MEMBER_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(&format!("{MEMBER_ROUTE}/role"), put(set_member_role_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
