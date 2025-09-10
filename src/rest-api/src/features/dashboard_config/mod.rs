// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::middleware::from_extractor_with_state;
use axum::routing::MethodRouter;
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/dashboard/config",
            MethodRouter::new().get(get_dashboard_config_route),
        )
        .route(
            "/v1/dashboard/config/url",
            MethodRouter::new().get(get_dashboard_url_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

mod routes {
    use axum::{extract::State, Json};
    use service::{app_config::DashboardConfig, models::Url};

    use crate::AppState;

    pub async fn get_dashboard_config_route(
        State(AppState { app_config, .. }): State<AppState>,
    ) -> Json<DashboardConfig> {
        Json(app_config.read().unwrap().dashboard.clone())
    }

    pub(super) async fn get_dashboard_url_route(
        State(AppState { app_config, .. }): State<AppState>,
    ) -> Json<Url> {
        Json(app_config.read().unwrap().dashboard_url().to_owned())
    }
}
