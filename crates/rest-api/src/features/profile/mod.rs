// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_avatar;
mod set_member_nickname;

use axum::middleware::from_extractor_with_state;
use axum::routing::put;

use crate::AppState;

pub use self::set_member_avatar::*;
pub use self::set_member_nickname::*;

use super::auth::guards::Authenticated;
use super::members::MEMBER_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            MEMBER_ROUTE,
            axum::Router::new()
                .route("/avatar", put(set_member_avatar_route))
                .route("/nickname", put(set_member_nickname_route)),
        )
        .route_layer(from_extractor_with_state::<Authenticated, _>(
            app_state.clone(),
        ))
        .with_state(app_state)
}
