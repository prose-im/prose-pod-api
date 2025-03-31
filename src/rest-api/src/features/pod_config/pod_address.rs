// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::StatusCode, response::NoContent, Json};
use axum_extra::either::Either;
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

pub async fn set_pod_address_route(
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
            address: Some(NetworkAddressCreateForm {
                ipv4: req.ipv4,
                ipv6: req.ipv6,
                hostname: req.hostname,
            }),
            ..Default::default()
        },
    )
    .await?;

    let res = PodConfig::from(model).address.unwrap();
    Ok(Json(res))
}

pub async fn get_pod_address_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Either<Json<NetworkAddress>, NoContent>, Error> {
    Ok(match PodConfigRepository::get_pod_address(&db).await? {
        Some(address) => Either::E1(Json(address)),
        None => Either::E2(NoContent),
    })
}

#[derive(Debug, thiserror::Error)]
#[error("Prose Pod address not initialized.")]
pub struct PodAddressNotInitialized;
impl ErrorCode {
    pub const POD_ADDRESS_NOT_INITIALIZED: Self = Self {
        value: "pod_address_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}
impl HttpApiError for PodAddressNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::POD_ADDRESS_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {POD_CONFIG_ROUTE}` to initialize it.",
        )]
    }
}
