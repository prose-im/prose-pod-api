// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod forms;
pub mod routes;

pub use forms::*;
pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        invite_member,
        get_invitations,
        get_invitation,
        get_invitation_by_token,
        invitation_accept,
        invitation_reject,
        invitation_resend,
        invitation_cancel,
    ]
}
