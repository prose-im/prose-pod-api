// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{http::HeaderValue, Json};
use serde::{Deserialize, Serialize};
use service::{
    init::{InitService, InitWorkspaceError},
    secrets::SecretsStore,
    server_config::ServerConfig,
    workspace::{Workspace, WorkspaceServiceError},
    xmpp::XmppServiceInner,
    AppConfig,
};

use crate::{error::prelude::*, features::workspace_details::WORKSPACE_ROUTE, responders::Created};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

pub async fn init_workspace_route(
    init_service: InitService,
    app_config: AppConfig,
    secrets_store: SecretsStore,
    xmpp_service: XmppServiceInner,
    server_config: ServerConfig,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<Workspace>, Error> {
    let workspace = init_service
        .init_workspace(
            Arc::new(app_config),
            Arc::new(secrets_store),
            Arc::new(xmpp_service),
            &server_config,
            req.clone(),
        )
        .await?;

    let response = Workspace {
        name: req.name,
        accent_color: workspace.accent_color,
        icon: None,
    };

    let resource_uri = WORKSPACE_ROUTE;
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: response,
    })
}

// ERRORS

impl ErrorCode {
    pub const WORKSPACE_NOT_INITIALIZED: Self = Self {
        value: "workspace_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
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
            "Call `PUT {WORKSPACE_ROUTE}` to initialize it.",
        )]
    }
}

impl CustomErrorCode for InitWorkspaceError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceAlreadyInitialized => ErrorCode::WORKSPACE_ALREADY_INITIALIZED,
            Self::XmppAccountNotInitialized => ErrorCode::SERVER_CONFIG_NOT_INITIALIZED,
            Self::CouldNotSetWorkspaceVCard(err) => err.code(),
        }
    }
}
impl_into_error!(InitWorkspaceError);

impl HttpApiError for WorkspaceServiceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotInitialized(_) => WorkspaceNotInitialized.code(),
            Self::XmppServiceError(err) => err.code(),
        }
    }
    fn message(&self) -> String {
        format!("WorkspaceServiceError: {self}")
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::WorkspaceNotInitialized(_) => WorkspaceNotInitialized.recovery_suggestions(),
            _ => vec![],
        }
    }
}

// BOILERPLATE

impl Into<Workspace> for InitWorkspaceRequest {
    fn into(self) -> Workspace {
        Workspace {
            name: self.name,
            accent_color: self.accent_color,
            icon: None,
        }
    }
}
