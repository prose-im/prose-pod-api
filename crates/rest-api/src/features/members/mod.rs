// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod delete_member;
mod enrich_members;
mod get_member;
mod get_members;
mod guards;
mod model;

use axum::routing::{get, MethodRouter};
use axum_extra::handler::HandlerCallWithExtractors as _;

use crate::util::content_type_or::*;

pub use self::delete_member::*;
pub use self::enrich_members::*;
pub use self::get_member::*;
pub use self::get_members::*;
pub use self::model::*;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route(
            "/v1/enrich-members",
            get(
                with_content_type::<TextEventStream, _>(enrich_members_stream_route)
                    .or(enrich_members_route),
            ),
        )
        .route("/v1/members", get(get_members_route))
        .route(
            "/v1/members/:jid",
            MethodRouter::new()
                .get(get_member_route)
                .delete(delete_member_route),
        )
}
