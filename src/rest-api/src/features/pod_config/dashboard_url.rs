// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::StatusCode, Json};
use service::models::Url;

use crate::{
    error::{Error, ErrorCode, HttpApiError, LogLevel},
    pod_config_routes, AppState,
};

use super::{check_url_has_no_path, POD_CONFIG_ROUTE};

pod_config_routes!(
    key: dashboard_url, type: Option<Url>,
    get: get_dashboard_url_route using get_dashboard_url,
);

pub async fn set_dashboard_url_route(
    State(app_state): State<AppState>,
    req: Json<Option<Url>>,
) -> Result<Json<Option<Url>>, Error> {
    pod_config_routes!(
        key: dashboard_url, type: Option<Url>,
        set: set_dashboard_url_route, validate: {
            check_url_has_no_path(&dashboard_url)?;
        },
    );

    let Json(dashboard_url) = set_dashboard_url_route(State(app_state.clone()), req).await?;

    if let Some(ref dashboard_url) = dashboard_url {
        (app_state.cors_config.allowed_origins.write().unwrap()).insert(dashboard_url.clone());
    }

    Ok(Json(dashboard_url))
}

#[derive(Debug, thiserror::Error)]
#[error("Prose Pod Dashboard URL not initialized.")]
pub struct DashboardUrlNotInitialized;
impl ErrorCode {
    pub const DASHBOARD_URL_NOT_INITIALIZED: Self = Self {
        value: "dashboard_url_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}
impl HttpApiError for DashboardUrlNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::DASHBOARD_URL_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {POD_CONFIG_ROUTE}` to initialize it.",
        )]
    }
}
