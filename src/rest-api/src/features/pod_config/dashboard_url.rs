// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::StatusCode;
use service::models::Url;

use crate::{
    error::{ErrorCode, HttpApiError, LogLevel},
    pod_config_routes,
};

use super::{check_url_has_no_path, POD_CONFIG_ROUTE};

pod_config_routes!(
    dashboard_url,
    Option<Url>,
    get: get_dashboard_url_route,
    get_fn: get_dashboard_url,
    set: set_dashboard_url_route,
    validate_set: { check_url_has_no_path(&dashboard_url)?; },
);

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
