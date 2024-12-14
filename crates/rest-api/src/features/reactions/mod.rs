// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod routes;

use axum::routing::{delete, get, post, put};

pub use self::routes::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_workspace_reactions_route,
        add_workspace_reaction_route,
        get_workspace_reaction_route,
        edit_workspace_reaction_route,
        delete_workspace_reaction_route,
    ]
}

pub(super) fn router() -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/workspace/reactions",
            get(get_workspace_reactions_route_axum),
        )
        .route(
            "/v1/workspace/reactions",
            post(add_workspace_reaction_route_axum),
        )
        .route(
            "/v1/workspace/reactions/:reaction_id",
            get(get_workspace_reaction_route_axum),
        )
        .route(
            "/v1/workspace/reactions/:reaction_id",
            put(edit_workspace_reaction_route_axum),
        )
        .route(
            "/v1/workspace/reactions/:reaction_id",
            delete(delete_workspace_reaction_route_axum),
        )
}
