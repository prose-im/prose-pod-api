// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_mfa;

use axum::routing::put;

pub use self::set_member_mfa::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![set_member_mfa_route]
}

pub(super) fn router() -> axum::Router {
    axum::Router::new().route("/v1/members/:jid/mfa", put(set_member_mfa_route_axum))
}
