// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod reactions;
pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![
        reactions::routes(),
        self::_routes(),
    ]
    .concat()
}

fn _routes() -> Vec<Route> {
    routes![
        get_workspace,
        get_workspace_name,
        set_workspace_name,
        get_workspace_icon,
        set_workspace_icon,
        get_workspace_details_card,
        set_workspace_details_card,
        get_workspace_accent_color,
        set_workspace_accent_color,
    ]
}
