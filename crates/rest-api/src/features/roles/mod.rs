// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_role;

pub use set_member_role::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![set_member_role_route]
}
