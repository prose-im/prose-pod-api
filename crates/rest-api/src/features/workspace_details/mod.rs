// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_workspace;
mod guards;
mod workspace_accent_color;
mod workspace_icon;
mod workspace_name;
mod workspace_vcard;

use axum::routing::{get, put};

pub use self::get_workspace::*;
pub use self::workspace_accent_color::*;
pub use self::workspace_icon::*;
pub use self::workspace_name::*;
pub use self::workspace_vcard::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_workspace_route,
        get_workspace_accent_color_route,
        set_workspace_accent_color_route,
        get_workspace_icon_route,
        set_workspace_icon_route,
        get_workspace_name_route,
        set_workspace_name_route,
        get_workspace_details_card_route,
        set_workspace_details_card_route,
    ]
}

pub(super) fn router() -> axum::Router {
    axum::Router::new()
        .route("/v1/workspace", get(get_workspace_route_axum))
        .route(
            "/v1/workspace/accent-color",
            get(get_workspace_accent_color_route_axum),
        )
        .route(
            "/v1/workspace/accent-color",
            put(set_workspace_accent_color_route_axum),
        )
        .route("/v1/workspace/icon", get(get_workspace_icon_route_axum))
        .route("/v1/workspace/icon", put(set_workspace_icon_route_axum))
        .route("/v1/workspace/name", get(get_workspace_name_route_axum))
        .route("/v1/workspace/name", put(set_workspace_name_route_axum))
        .route(
            "/v1/workspace/details-card",
            get(get_workspace_details_card_route_axum),
        )
        .route(
            "/v1/workspace/details-card",
            put(set_workspace_details_card_route_axum),
        )
}
