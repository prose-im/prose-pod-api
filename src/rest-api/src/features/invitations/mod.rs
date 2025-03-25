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

use axum::middleware::from_extractor_with_state;
use axum::routing::*;

use crate::AppState;

pub use self::forms::*;
pub use self::get_invitation::*;
pub use self::get_invitations::*;
pub use self::invitation_actions::*;
pub use self::invite_member::*;
pub use self::model::*;

use super::auth::guards::IsAdmin;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            "/v1/invitations",
            axum::Router::new()
                .route(
                    "/",
                    MethodRouter::new()
                        .post(invite_member_route)
                        .get(get_invitations_route),
                )
                .route(
                    "/{invitation_id}",
                    MethodRouter::new()
                        .get(get_invitation_route)
                        .delete(invitation_cancel_route),
                )
                .route("/{invitation_id}/resend", post(invitation_resend_route))
                .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
        )
        .nest(
            "/v1/invitation-tokens",
            axum::Router::new()
                .route("/{token}/details", get(get_invitation_by_token_route))
                .route("/{token}/accept", put(invitation_accept_route))
                .route("/{token}/reject", put(invitation_reject_route)),
        )
        .with_state(app_state)
}
