// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::StatusCode, response::NoContent, Json};
use axum_extra::either::Either;
use serde::{Deserialize, Serialize};
use service::{
    models::Url,
    pod_config::{PodConfigRepository, PodConfigUpdateForm},
};

use crate::{
    error::{self, Error, ErrorCode, HttpApiError, LogLevel},
    AppState,
};

use super::{check_url_has_no_path, PodConfigNotInitialized, POD_CONFIG_ROUTE};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetDashboardUrlRequest {
    #[serde(default, deserialize_with = "crate::forms::deserialize_some")]
    pub dashboard_url: Option<Option<Url>>,
}

pub async fn set_dashboard_url_route(
    State(AppState { db, .. }): State<AppState>,
    Json(req): Json<SetDashboardUrlRequest>,
) -> Result<Json<GetDashboardUrlResponse>, Error> {
    let Some(dashboard_url) = req.dashboard_url else {
        return Err(Error::from(error::HTTPStatus {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: "`dashboard_url` is required.".to_string(),
        }));
    };
    check_url_has_no_path(&dashboard_url)?;

    if !PodConfigRepository::is_initialized(&db).await? {
        return Err(Error::from(PodConfigNotInitialized));
    }

    let model = PodConfigRepository::set(
        &db,
        PodConfigUpdateForm {
            dashboard_url: Some(dashboard_url),
            ..Default::default()
        },
    )
    .await?;

    Ok(Json(GetDashboardUrlResponse {
        dashboard_url: model.dashboard_url,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDashboardUrlResponse {
    pub dashboard_url: Option<Url>,
}

pub async fn get_dashboard_url_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Either<Json<GetDashboardUrlResponse>, NoContent>, Error> {
    Ok(match PodConfigRepository::get_dashboard_url(&db).await? {
        Some(url) => Either::E1(Json(GetDashboardUrlResponse {
            dashboard_url: Some(url),
        })),
        None => Either::E2(NoContent),
    })
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
