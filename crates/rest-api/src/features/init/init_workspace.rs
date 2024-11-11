// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use service::{
    init::{InitController, InitWorkspaceError, WorkspaceCreateForm},
    secrets::SecretsStore,
    server_config::ServerConfig,
    workspace::WorkspaceControllerError,
    xmpp::XmppServiceInner,
    AppConfig,
};

use crate::{error::prelude::*, guards::LazyGuard, responders::Created};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceResponse {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

#[put("/v1/workspace", format = "json", data = "<req>")]
pub async fn init_workspace_route<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    app_config: &State<AppConfig>,
    secrets_store: &State<SecretsStore>,
    xmpp_service: &State<XmppServiceInner>,
    server_config: LazyGuard<ServerConfig>,
    req: Json<InitWorkspaceRequest>,
) -> Created<InitWorkspaceResponse> {
    let init_controller = init_controller.inner?;
    let server_config = server_config.inner?;
    let req = req.into_inner();

    let workspace = init_controller
        .init_workspace(
            app_config,
            secrets_store,
            xmpp_service,
            &server_config,
            req.clone(),
        )
        .await?;

    let response = InitWorkspaceResponse {
        name: req.name,
        accent_color: workspace.accent_color,
    };

    let resource_uri = uri!(crate::features::workspace_details::get_workspace_route).to_string();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

// ERRORS

impl ErrorCode {
    pub const WORKSPACE_NOT_INITIALIZED: Self = Self {
        value: "workspace_not_initialized",
        http_status: Status::BadRequest,
        log_level: LogLevel::Warn,
    };
    pub const WORKSPACE_ALREADY_INITIALIZED: Self = Self {
        value: "workspace_already_initialized",
        http_status: Status::Conflict,
        log_level: LogLevel::Info,
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Workspace not initialized. Call `PUT {}` to initialize it.", uri!(crate::features::init::init_workspace_route))]
pub struct WorkspaceNotInitialized;
impl HttpApiError for WorkspaceNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::WORKSPACE_NOT_INITIALIZED
    }
}

impl CustomErrorCode for InitWorkspaceError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceAlreadyInitialized => ErrorCode::WORKSPACE_ALREADY_INITIALIZED,
            Self::XmppAccountNotInitialized => ErrorCode::SERVER_CONFIG_NOT_INITIALIZED,
            Self::CouldNotSetWorkspaceName(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InitWorkspaceError);

impl CustomErrorCode for WorkspaceControllerError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotInitialized => ErrorCode::WORKSPACE_NOT_INITIALIZED,
            Self::XmppServiceError(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(WorkspaceControllerError);

// BOILERPLATE

impl Into<WorkspaceCreateForm> for InitWorkspaceRequest {
    fn into(self) -> WorkspaceCreateForm {
        WorkspaceCreateForm {
            name: self.name,
            accent_color: Some(self.accent_color),
        }
    }
}