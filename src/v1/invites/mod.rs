// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        invite_member,
        get_invites,
        get_invite,
        get_invite_by_token,
        invite_accept,
        invite_reject,
        invite_resend,
        invite_cancel,
        cancel_invite,
    ]
}
