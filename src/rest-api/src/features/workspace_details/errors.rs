// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    init::InitWorkspaceError,
    workspace::{
        workspace_controller::SetWorkspaceIconError, GetWorkspaceError, WorkspaceNotInitialized,
        WorkspaceServiceInitError,
    },
};

use crate::error::prelude::*;

use super::WORKSPACE_ROUTE;

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

impl HttpApiError for InitWorkspaceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceAlreadyInitialized => ErrorCode::WORKSPACE_ALREADY_INITIALIZED,
            Self::XmppAccountNotInitialized => ErrorCode::SERVER_CONFIG_NOT_INITIALIZED,
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for GetWorkspaceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotInitialized(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::WorkspaceNotInitialized(err) => err.recovery_suggestions(),
            Self::Internal(_) => vec![],
        }
    }
}

impl HttpApiError for SetWorkspaceIconError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::BadImageDataFormat(_) => ErrorCode::BAD_REQUEST,
            Self::UnsupportedMediaType => ErrorCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for WorkspaceServiceInitError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceXmppAccountNotInitialized => ErrorCode::SERVER_CONFIG_NOT_INITIALIZED,
        }
    }
}
