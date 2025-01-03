// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_workspace;
mod guards;
mod workspace_accent_color;
mod workspace_icon;
mod workspace_name;

use axum::routing::{get, put};

pub use self::get_workspace::*;
pub use self::workspace_accent_color::*;
pub use self::workspace_icon::*;
pub use self::workspace_name::*;

use super::init::WORKSPACE_ROUTE;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route(WORKSPACE_ROUTE, get(get_workspace_route))
        .route(
            "/v1/workspace/accent-color",
            get(get_workspace_accent_color_route),
        )
        .route(
            "/v1/workspace/accent-color",
            put(set_workspace_accent_color_route),
        )
        .route("/v1/workspace/icon", get(get_workspace_icon_route))
        .route("/v1/workspace/icon", put(set_workspace_icon_route))
        .route("/v1/workspace/name", get(get_workspace_name_route))
        .route("/v1/workspace/name", put(set_workspace_name_route))
}
