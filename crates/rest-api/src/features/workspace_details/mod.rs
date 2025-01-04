// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_workspace;
mod guards;
mod workspace_accent_color;
mod workspace_icon;
mod workspace_name;

use axum::routing::MethodRouter;

use crate::AppState;

pub use self::get_workspace::*;
pub use self::workspace_accent_color::*;
pub use self::workspace_icon::*;
pub use self::workspace_name::*;

use super::init::WORKSPACE_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            WORKSPACE_ROUTE,
            axum::Router::new()
                .route("/", MethodRouter::new().get(get_workspace_route))
                .route(
                    "/accent-color",
                    MethodRouter::new()
                        .get(get_workspace_accent_color_route)
                        .put(set_workspace_accent_color_route),
                )
                .route(
                    "/icon",
                    MethodRouter::new()
                        .get(get_workspace_icon_route)
                        .put(set_workspace_icon_route),
                )
                .route(
                    "/name",
                    MethodRouter::new()
                        .get(get_workspace_name_route)
                        .put(set_workspace_name_route),
                ),
        )
        .with_state(app_state)
}
