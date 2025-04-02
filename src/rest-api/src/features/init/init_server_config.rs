// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{http::HeaderValue, Json};
use serde::{Deserialize, Serialize};
use service::{
    auth::AuthService,
    init::{InitServerConfigError, InitService},
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigCreateForm},
    xmpp::{JidDomain, ServerCtl},
    AppConfig,
};

use crate::{error::prelude::*, features::init::SERVER_CONFIG_ROUTE, responders::Created};

#[derive(Serialize, Deserialize)]
pub struct InitServerConfigRequest {
    /// XMPP server domain (e.g. `crisp.chat`).
    /// This is what will appear in JIDs (e.g. `valerian@`**`crisp.chat`**).
    pub domain: JidDomain,
}

pub async fn init_server_config_route(
    init_service: InitService,
    server_ctl: ServerCtl,
    app_config: AppConfig,
    auth_service: AuthService,
    secrets_store: SecretsStore,
    Json(req): Json<InitServerConfigRequest>,
) -> Result<Created<ServerConfig>, Error> {
    let server_config = init_service
        .init_server_config(&server_ctl, &app_config, &auth_service, &secrets_store, req)
        .await?;

    let resource_uri = SERVER_CONFIG_ROUTE;
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: server_config,
    })
}

// ERRORS

impl ErrorCode {
    pub const SERVER_CONFIG_NOT_INITIALIZED: Self = Self {
        value: "server_config_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
    pub const SERVER_CONFIG_ALREADY_INITIALIZED: Self = Self {
        value: "server_config_already_initialized",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
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
            "Call `PUT {SERVER_CONFIG_ROUTE}` to initialize it.",
        )]
    }
}

impl CustomErrorCode for InitServerConfigError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CouldNotInitServerConfig(err) => err.code(),
            Self::CouldNotRegisterOAuth2Client(err) => err.code(),
            Self::CouldNotCreateServiceAccount(err) => err.code(),
            InitServerConfigError::CouldNotAddWorkspaceToTeam(err) => err.code(),
        }
    }
}
impl_into_error!(InitServerConfigError);

// BOILERPLATE

impl Into<ServerConfigCreateForm> for InitServerConfigRequest {
    fn into(self) -> ServerConfigCreateForm {
        ServerConfigCreateForm {
            domain: self.domain.to_owned(),
        }
    }
}
