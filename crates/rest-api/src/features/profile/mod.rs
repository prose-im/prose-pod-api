// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_avatar;
mod set_member_nickname;

use axum::routing::put;

pub use self::set_member_avatar::*;
pub use self::set_member_nickname::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        set_member_avatar_route,
        set_member_nickname_route,
    ]
}

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/v1/members/:jid/avatar", put(set_member_avatar_route_axum))
        .route(
            "/v1/members/:jid/nickname",
            put(set_member_nickname_route_axum),
        )
}
