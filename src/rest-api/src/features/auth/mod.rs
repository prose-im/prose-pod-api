// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;
pub mod extractors;
mod routes;

use axum::{middleware::from_extractor_with_state, routing::*};
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

use super::members::MEMBER_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(&format!("{MEMBER_ROUTE}/role"), put(set_member_role_route))
        .route(
            &format!("{MEMBER_ROUTE}/password"),
            delete(request_password_reset_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .route("/v1/login", post(login_route))
        .route(
            "/v1/password-reset-tokens/{token}/use",
            put(reset_password_route),
        )
        .with_state(app_state)
}
