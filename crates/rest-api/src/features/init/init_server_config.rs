// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use service::{
    auth::AuthService,
    init::{InitController, InitServerConfigError},
    pod_config::PodAddressError,
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigCreateForm},
    xmpp::{JidDomain, ServerCtl},
    AppConfig,
};

use crate::{error::prelude::*, guards::LazyGuard, responders::Created};

#[derive(Serialize, Deserialize)]
pub struct InitServerConfigRequest {
    /// XMPP server domain (e.g. `crisp.chat`).
    /// This is what will appear in JIDs (e.g. `valerian@`**`crisp.chat`**).
    pub domain: JidDomain,
}

#[put("/v1/server/config", format = "json", data = "<req>")]
pub async fn init_server_config_route<'r>(
    init_controller: LazyGuard<InitController>,
    server_ctl: &State<ServerCtl>,
    app_config: &State<AppConfig>,
    auth_service: &State<AuthService>,
    secrets_store: &State<SecretsStore>,
    req: Json<InitServerConfigRequest>,
) -> Created<ServerConfig> {
    let init_controller = init_controller.inner?;
    let form = req.into_inner();

    let server_config = init_controller
        .init_server_config(server_ctl, app_config, auth_service, secrets_store, form)
        .await?;

    let resource_uri = uri!(crate::features::server_config::get_server_config_route).to_string();
    Ok(status::Created::new(resource_uri).body(server_config.into()))
}

// ERRORS

impl ErrorCode {
    pub const SERVER_CONFIG_NOT_INITIALIZED: Self = Self {
        value: "server_config_not_initialized",
        http_status: Status::BadRequest,
        log_level: LogLevel::Warn,
    };
    pub const SERVER_CONFIG_ALREADY_INITIALIZED: Self = Self {
        value: "server_config_already_initialized",
        http_status: Status::Conflict,
        log_level: LogLevel::Info,
    };
    pub const POD_ADDRESS_NOT_INITIALIZED: Self = Self {
        value: "pod_address_not_initialized",
        http_status: Status::BadRequest,
        log_level: LogLevel::Warn,
    };
}

#[derive(Debug, thiserror::Error)]
#[error("XMPP server not initialized.")]
pub struct ServerConfigNotInitialized;
impl HttpApiError for ServerConfigNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::SERVER_CONFIG_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {}` to initialize it.",
            uri!(crate::features::init::init_server_config_route)
        )]
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Prose Pod address not initialized.")]
pub struct PodAddressNotInitialized;
impl HttpApiError for PodAddressNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::POD_ADDRESS_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {}` to initialize it.",
            uri!(crate::features::pod_config::set_pod_address_route)
        )]
    }
}

impl CustomErrorCode for InitServerConfigError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CouldNotInitServerConfig(err) => err.code(),
            Self::CouldNotRegisterOAuth2Client(err) => err.code(),
            Self::CouldNotCreateServiceAccount(err) => err.code(),
        }
    }
}
impl_into_error!(InitServerConfigError);

impl CustomErrorCode for PodAddressError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::PodAddressNotInitialized => ErrorCode::POD_ADDRESS_NOT_INITIALIZED,
            Self::InvalidData(_) => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(PodAddressError);

// BOILERPLATE

impl Into<ServerConfigCreateForm> for InitServerConfigRequest {
    fn into(self) -> ServerConfigCreateForm {
        ServerConfigCreateForm {
            domain: self.domain.to_owned(),
        }
    }
}
