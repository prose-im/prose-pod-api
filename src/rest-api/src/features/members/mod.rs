// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;
mod extractors;
mod routes;

use axum::{
    middleware::from_extractor_with_state,
    routing::{delete, get, head},
};
use service::auth::{Authenticated, IsAdmin};

use crate::AppState;

pub use self::routes::*;

pub(crate) const MEMBERS_ROUTE: &'static str = "/v1/members";
pub(crate) const MEMBER_ROUTE: &'static str = "/v1/members/{jid}";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/enrich-members", get(enrich_members_route))
        .route(MEMBERS_ROUTE, get(get_members_route))
        .route(MEMBER_ROUTE, get(get_member_route))
        .nest(
            MEMBER_ROUTE,
            axum::Router::new().route(
                "/",
                delete(delete_member_route)
                    .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
            ),
        )
        .route_layer(from_extractor_with_state::<Authenticated, _>(
            app_state.clone(),
        ))
        .route(MEMBERS_ROUTE, head(head_members))
        .with_state(app_state)
}
