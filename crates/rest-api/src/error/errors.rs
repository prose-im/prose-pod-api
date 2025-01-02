// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    members::{UserCreateError, UserDeleteError},
    notifications::notifier,
    sea_orm,
    xmpp::{server_ctl, server_manager, xmpp_service, CreateServiceAccountError},
    MutationError,
};

use super::prelude::*;

impl ErrorCode {
    pub const NOT_IMPLEMENTED: Self = Self {
        value: "not_implemented",
        http_status: StatusCode::NOT_IMPLEMENTED,
        log_level: LogLevel::Error,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Feature not implemented yet: {0}")]
pub struct NotImplemented(pub &'static str);
impl HttpApiError for NotImplemented {
    fn code(&self) -> ErrorCode {
        ErrorCode::NOT_IMPLEMENTED
    }
}

impl ErrorCode {
    pub const INTERNAL_SERVER_ERROR: Self = Self {
        value: "internal_server_error",
        http_status: StatusCode::INTERNAL_SERVER_ERROR,
        log_level: LogLevel::Error,
    };
}
/// Internal server error.
/// Use it only when a nearly-impossible code path is taken.
#[derive(Debug, thiserror::Error)]
#[error("Internal server error: {0}")]
pub struct InternalServerError(pub String);
impl HttpApiError for InternalServerError {
    fn code(&self) -> ErrorCode {
        ErrorCode::INTERNAL_SERVER_ERROR
    }
}

impl ErrorCode {
    pub const UNAUTHORIZED: Self = Self {
        value: "unauthorized",
        http_status: StatusCode::UNAUTHORIZED,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Unauthorized: {0}")]
pub struct Unauthorized(pub String);
impl HttpApiError for Unauthorized {
    fn code(&self) -> ErrorCode {
        ErrorCode::UNAUTHORIZED
    }
    fn http_headers(&self) -> Vec<(String, String)> {
        vec![(
            "WWW-Authenticate".into(),
            r#"Bearer realm="Admin only area", charset="UTF-8""#.into(),
        )]
    }
}

impl ErrorCode {
    pub const FORBIDDEN: Self = Self {
        value: "forbidden",
        http_status: StatusCode::FORBIDDEN,
        log_level: LogLevel::Warn,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Forbidden: {0}")]
pub struct Forbidden(pub String);
impl HttpApiError for Forbidden {
    fn code(&self) -> ErrorCode {
        ErrorCode::FORBIDDEN
    }
}

impl ErrorCode {
    pub const DATABASE_ERROR: Self = Self {
        value: "database_error",
        http_status: StatusCode::INTERNAL_SERVER_ERROR,
        log_level: LogLevel::Error,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Unknown database error")]
pub struct UnknownDbErr;
impl HttpApiError for UnknownDbErr {
    fn code(&self) -> ErrorCode {
        ErrorCode::DATABASE_ERROR
    }
}

impl ErrorCode {
    pub const BAD_REQUEST: Self = Self {
        value: "bad_request",
        http_status: StatusCode::BAD_REQUEST,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Bad request: {reason}")]
pub struct BadRequest {
    pub reason: String,
}
impl HttpApiError for BadRequest {
    fn code(&self) -> ErrorCode {
        ErrorCode::BAD_REQUEST
    }
}

impl ErrorCode {
    pub const NOT_FOUND: Self = Self {
        value: "not_found",
        http_status: StatusCode::NOT_FOUND,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Not found: {reason}")]
pub struct NotFound {
    pub reason: String,
}
impl HttpApiError for NotFound {
    fn code(&self) -> ErrorCode {
        ErrorCode::NOT_FOUND
    }
}

impl ErrorCode {
    pub fn unknown(status: StatusCode) -> Self {
        Self {
            value: "unknown",
            http_status: status,
            log_level: if (500..600).contains(&status.as_u16()) {
                // Server error
                LogLevel::Error
            } else {
                // Client error
                LogLevel::Warn
            },
        }
    }
}
/// HTTP status (used by the [default catcher](https://rocket.rs/guide/v0.5/requests/#default-catchers)
/// to change the output format).
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HTTPStatus(pub StatusCode);
impl HttpApiError for HTTPStatus {
    fn code(&self) -> ErrorCode {
        ErrorCode::unknown(self.0)
    }
}

impl_into_error!(sea_orm::DbErr, ErrorCode::DATABASE_ERROR);

impl HttpApiError for server_ctl::Error {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Unauthorized(_) => ErrorCode::UNAUTHORIZED,
            Self::Forbidden(_) => ErrorCode::FORBIDDEN,
            Self::UnexpectedResponse(err) => err.code(),
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        std::format!("server_ctl::Error: {self}")
    }

    fn debug_info(&self) -> Option<serde_json::Value> {
        match self {
            Self::UnexpectedResponse(err) => err.debug_info(),
            _ => None,
        }
    }
}

impl_into_error!(xmpp_service::Error, ErrorCode::INTERNAL_SERVER_ERROR);

impl_into_error!(notifier::Error, ErrorCode::INTERNAL_SERVER_ERROR);

impl CustomErrorCode for MutationError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::EntityNotFound { .. } => ErrorCode::NOT_FOUND,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(MutationError);

impl CustomErrorCode for UserCreateError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(UserCreateError);

impl CustomErrorCode for UserDeleteError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(UserDeleteError);

impl CustomErrorCode for server_manager::Error {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::ServerConfigAlreadyInitialized => ErrorCode::SERVER_CONFIG_ALREADY_INITIALIZED,
            Self::ServerCtl(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(server_manager::Error);

impl CustomErrorCode for CreateServiceAccountError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateXmppAccount(err) => err.code(),
            Self::CouldNotLogIn(_) => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(CreateServiceAccountError);

impl HttpApiError for service::errors::UnexpectedHttpResponse {
    fn code(&self) -> ErrorCode {
        ErrorCode::INTERNAL_SERVER_ERROR
    }

    fn message(&self) -> String {
        std::format!("{self}")
    }

    fn debug_info(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self)
            .inspect_err(|err| tracing::error!("Could not serialize error `{self}`: {err}"))
            .ok()
    }
}
