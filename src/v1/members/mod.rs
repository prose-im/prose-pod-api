// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod models;
pub mod routes;

pub use models::*;
pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        get_members,
        enrich_members,
        get_member,
        set_member_role,
        set_member_mfa,
        logout_member,
        set_member_nickname,
        set_member_avatar,
    ]
}
