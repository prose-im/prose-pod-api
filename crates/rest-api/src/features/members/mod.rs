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

pub use delete_member::*;
pub use enrich_members::*;
pub use get_member::*;
pub use get_members::*;
pub use model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        enrich_members_route,
        enrich_members_stream_route,
        get_member_route,
        get_members_route,
        delete_member_route,
    ]
}
