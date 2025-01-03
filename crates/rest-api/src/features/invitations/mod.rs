// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
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

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route(
            "/v1/invitations",
            MethodRouter::new()
                .post(invite_member_route)
                .get(get_invitations_route),
        )
        .route(
            "/v1/invitations/:invitation_id",
            get(get_invitation_route),
        )
        .route(
            "/v1/invitations/:invitation_id/resend",
            put(invitation_resend_route),
        )
        .route(
            "/v1/invitations/:invitation_id/cancel",
            put(invitation_cancel_route),
        )
        .route(
            "/v1/invitations-tokens/:token/details",
            get(get_invitation_by_token_route),
        )
        .route(
            "/v1/invitations-tokens/:token/accept",
            put(invitation_accept_route),
        )
        .route(
            "/v1/invitations-tokens/:token/reject",
            put(invitation_reject_route),
        )
}
