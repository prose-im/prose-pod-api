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

pub use forms::*;
pub use get_invitation::*;
pub use get_invitations::*;
pub use invitation_actions::*;
pub use invite_member::*;
pub use model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        invite_member_route,
        get_invitations_route,
        get_invitation_route,
        get_invitation_by_token_route,
        invitation_accept_route,
        invitation_reject_route,
        invitation_resend_route,
        invitation_cancel_route,
    ]
}
