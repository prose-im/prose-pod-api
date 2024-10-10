// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        get_workspace_reactions,
        add_workspace_reaction,
        get_workspace_reaction,
        edit_workspace_reaction,
        delete_workspace_reaction,
    ]
}
