// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_role;

use axum::routing::put;

pub use self::set_member_role::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![set_member_role_route]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new().route("/v1/members/:jid/role", put(set_member_role_route_axum))
}
