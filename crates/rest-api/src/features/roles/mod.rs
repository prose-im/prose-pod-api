// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod set_member_role;

use axum::routing::put;

pub use self::set_member_role::*;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new().route("/v1/members/:jid/role", put(set_member_role_route))
}
