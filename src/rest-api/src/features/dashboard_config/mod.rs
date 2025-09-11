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
    use std::sync::Arc;

    use axum::{extract::State, Json};
    use service::{app_config::DashboardConfig, models::Url, AppConfig};

    pub async fn get_dashboard_config_route(
        State(ref app_config): State<Arc<AppConfig>>,
    ) -> Json<Arc<DashboardConfig>> {
        Json(app_config.dashboard.clone())
    }

    pub(super) async fn get_dashboard_url_route(
        State(ref app_config): State<Arc<AppConfig>>,
    ) -> Json<Url> {
        Json(app_config.dashboard_url().to_owned())
    }
}
