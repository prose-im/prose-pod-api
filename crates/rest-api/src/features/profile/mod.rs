// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_avatar;
mod set_member_nickname;

pub use set_member_avatar::*;
pub use set_member_nickname::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        set_member_avatar_route,
        set_member_nickname_route,
    ]
}
