// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod forms;
mod get_invitation;
mod get_invitations;
mod guards;
mod invitation_actions;
mod invite_member;
mod model;

use axum::routing::*;

pub use self::forms::*;
pub use self::get_invitation::*;
pub use self::get_invitations::*;
pub use self::invitation_actions::*;
pub use self::invite_member::*;
pub use self::model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        invite_member_route,
        get_invitations_route,
        get_invitation_route,
        get_invitation_token_details_route,
        invitation_accept_route,
        invitation_reject_route,
        invitation_resend_route,
        invitation_cancel_route,
    ]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new()
        .route(
            "/v1/invitations",
            MethodRouter::new()
                .post(invite_member_route_axum)
                .get(get_invitations_route_axum),
        )
        .route(
            "/v1/invitations/:invitation_id",
            get(get_invitation_route_axum),
        )
        .route(
            "/v1/invitations/:invitation_id/resend",
            put(invitation_resend_route_axum),
        )
        .route(
            "/v1/invitations/:invitation_id/cancel",
            put(invitation_cancel_route_axum),
        )
        .route(
            "/v1/invitations-tokens/:token/details",
            get(get_invitation_by_token_route_axum),
        )
        .route(
            "/v1/invitations-tokens/:token/accept",
            put(invitation_accept_route_axum),
        )
        .route(
            "/v1/invitations-tokens/:token/reject",
            put(invitation_reject_route_axum),
        )
}
