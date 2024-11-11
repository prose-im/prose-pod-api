// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod routes;

pub use routes::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_workspace_reactions_route,
        add_workspace_reaction_route,
        get_workspace_reaction_route,
        edit_workspace_reaction_route,
        delete_workspace_reaction_route,
    ]
}
