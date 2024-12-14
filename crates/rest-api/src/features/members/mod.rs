// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod delete_member;
mod enrich_members;
mod get_member;
mod get_members;
mod guards;
mod model;

use axum::routing::{delete, get};

pub use self::delete_member::*;
pub use self::enrich_members::*;
pub use self::get_member::*;
pub use self::get_members::*;
pub use self::model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        enrich_members_route,
        enrich_members_stream_route,
        get_member_route,
        get_members_route,
        delete_member_route,
    ]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/v1/enrich-members", get(enrich_members_route_axum))
        .route("/v1/enrich-members", get(enrich_members_stream_route_axum))
        .route("/v1/members/:jid", get(get_member_route_axum))
        .route("/v1/members", get(get_members_route_axum))
        .route("/v1/members/:jid", delete(delete_member_route_axum))
}
