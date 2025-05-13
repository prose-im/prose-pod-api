// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;
mod guards;
mod routes;

use axum::middleware::from_extractor_with_state;
use axum::routing::{get, patch, put, MethodRouter};
use axum_extra::handler::HandlerCallWithExtractors as _;
use service::auth::IsAdmin;

use crate::util::content_type_or::{with_accept, ApplictionJson};
use crate::AppState;

pub use self::routes::*;

use super::init::init_workspace_route;

pub const WORKSPACE_ROUTE: &'static str = "/v1/workspace";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            WORKSPACE_ROUTE,
            axum::Router::new()
                .route(
                    "/",
                    MethodRouter::new()
                        .put(init_workspace_route)
                        .head(is_workspace_initialized_route)
                        .get(get_workspace_route),
                )
                .route("/accent-color", get(get_workspace_accent_color_route))
                .route(
                    "/icon",
                    get(
                        with_accept::<ApplictionJson, _>(get_workspace_icon_json_route)
                            .or(get_workspace_icon_route),
                    ),
                )
                .route("/name", get(get_workspace_name_route))
                .merge(
                    axum::Router::new()
                        .route("/", patch(patch_workspace_route))
                        .route("/accent-color", put(set_workspace_accent_color_route))
                        .route("/icon", put(set_workspace_icon_route))
                        .route("/name", put(set_workspace_name_route))
                        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
                ),
        )
        .with_state(app_state)
}
