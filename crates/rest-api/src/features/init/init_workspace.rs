// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{http::HeaderValue, Json};
use rocket::{response::status, State};
use serde::{Deserialize, Serialize};
use service::{
    init::{InitService, InitWorkspaceError, WorkspaceCreateForm},
    secrets::SecretsStore,
    server_config::ServerConfig,
    workspace::WorkspaceServiceError,
    xmpp::XmppServiceInner,
    AppConfig,
};

use crate::{
    error::prelude::*,
    guards::LazyGuard,
    responders::{Created, RocketCreated},
};

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

#[rocket::put("/v1/workspace", format = "json", data = "<req>")]
pub async fn init_workspace_route<'r>(
    init_service: LazyGuard<InitService>,
    app_config: &State<AppConfig>,
    secrets_store: &State<SecretsStore>,
    xmpp_service: &State<XmppServiceInner>,
    server_config: LazyGuard<ServerConfig>,
    req: rocket::serde::json::Json<InitWorkspaceRequest>,
) -> RocketCreated<InitWorkspaceResponse> {
    let init_service = init_service.inner?;
    let server_config = server_config.inner?;
    let req = req.into_inner();

    let workspace = init_service
        .init_workspace(
            Arc::new(app_config.inner().clone()),
            Arc::new(secrets_store.inner().clone()),
            Arc::new(xmpp_service.inner().clone()),
            &server_config,
            req.clone(),
        )
        .await?;

    let response = InitWorkspaceResponse {
        name: req.name,
        accent_color: workspace.accent_color,
    };

    let resource_uri =
        rocket::uri!(crate::features::workspace_details::get_workspace_route).to_string();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

pub async fn init_workspace_route_axum(
    init_service: InitService,
    app_config: AppConfig,
    secrets_store: SecretsStore,
    xmpp_service: XmppServiceInner,
    server_config: ServerConfig,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<InitWorkspaceResponse>, Error> {
    let workspace = init_service
        .init_workspace(
            Arc::new(app_config),
            Arc::new(secrets_store),
            Arc::new(xmpp_service),
            &server_config,
            req.clone(),
        )
        .await?;

    let response = InitWorkspaceResponse {
        name: req.name,
        accent_color: workspace.accent_color,
    };

    let resource_uri = "/v1/workspace";
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: response,
    })
}

// ERRORS

impl ErrorCode {
    pub const WORKSPACE_NOT_INITIALIZED: Self = Self {
        value: "workspace_not_initialized",
        http_status: StatusCode::BAD_REQUEST,
        log_level: LogLevel::Warn,
    };
    pub const WORKSPACE_ALREADY_INITIALIZED: Self = Self {
        value: "workspace_already_initialized",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Workspace not initialized.")]
pub struct WorkspaceNotInitialized;
impl HttpApiError for WorkspaceNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::WORKSPACE_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {}` to initialize it.",
            rocket::uri!(crate::features::init::init_workspace_route)
        )]
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

impl HttpApiError for WorkspaceServiceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotInitialized => WorkspaceNotInitialized.code(),
            Self::XmppServiceError(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
    fn message(&self) -> String {
        format!("WorkspaceServiceError: {self}")
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::WorkspaceNotInitialized => WorkspaceNotInitialized.recovery_suggestions(),
            _ => vec![],
        }
    }
}

// BOILERPLATE

impl Into<WorkspaceCreateForm> for InitWorkspaceRequest {
    fn into(self) -> WorkspaceCreateForm {
        WorkspaceCreateForm {
            name: self.name,
            accent_color: Some(self.accent_color),
        }
    }
}
