// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::StatusCode, Json};
use service::pod_config::{
    NetworkAddress, NetworkAddressCreateForm, PodConfig, PodConfigRepository, PodConfigUpdateForm,
};

use crate::{
    error::{Error, ErrorCode, HttpApiError, LogLevel},
    AppState,
};

use super::{
    check_processable_network_address, PodConfigNotInitialized, SetNetworkAddressRequest,
    POD_CONFIG_ROUTE,
};

pub async fn set_dashboard_address_route(
    State(AppState { db, .. }): State<AppState>,
    Json(req): Json<SetNetworkAddressRequest>,
) -> Result<Json<NetworkAddress>, Error> {
    check_processable_network_address(&req)?;

    if !PodConfigRepository::is_initialized(&db).await? {
        return Err(Error::from(PodConfigNotInitialized));
    }

    let model = PodConfigRepository::set(
        &db,
        PodConfigUpdateForm {
            dashboard_address: Some(NetworkAddressCreateForm {
                ipv4: req.ipv4,
                ipv6: req.ipv6,
                hostname: req.hostname,
            }),
            ..Default::default()
        },
    )
    .await?;

    let res = PodConfig::from(model).dashboard_address.unwrap();
    Ok(Json(res))
}

pub async fn get_dashboard_address_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Json<NetworkAddress>, Error> {
    let Some(address) = PodConfigRepository::get(&db)
        .await?
        .and_then(|model| PodConfig::from(model).address)
    else {
        return Err(DashboardAddressNotInitialized.into());
    };

    Ok(address.into())
}

#[derive(Debug, thiserror::Error)]
#[error("Prose Pod Dashboard address not initialized.")]
pub struct DashboardAddressNotInitialized;
impl ErrorCode {
    pub const DASHBOARD_ADDRESS_NOT_INITIALIZED: Self = Self {
        value: "dashboard_address_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}
impl HttpApiError for DashboardAddressNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::DASHBOARD_ADDRESS_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {POD_CONFIG_ROUTE}` to initialize it.",
        )]
    }
}
