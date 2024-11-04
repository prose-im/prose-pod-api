// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};

use rocket::http::{ContentType, Header, Status};
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde_json::json;
use service::services::server_manager::CreateServiceAccountError;
use service::services::{
    auth_service, jwt_service, notifier, server_ctl, server_manager, user_service, xmpp_service,
};
use service::{sea_orm, MutationError};

pub mod prelude {
    pub use rocket::http::Status;

    pub use crate::{error, impl_into_error};

    pub use super::{CustomErrorCode, Error, ErrorCode, HttpApiError, LogLevel};
}

#[derive(Debug)]
pub struct ErrorCode {
    /// User-facing error code (a string for easier understanding).
    pub value: &'static str,
    /// HTTP status to return for this error.
    pub http_status: Status,
    pub log_level: LogLevel,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ErrorCode {
    pub const NOT_IMPLEMENTED: Self = Self {
        value: "not_implemented",
        http_status: Status::NotImplemented,
        log_level: LogLevel::Error,
    };
    pub const INTERNAL_SERVER_ERROR: Self = Self {
        value: "internal_server_error",
        http_status: Status::InternalServerError,
        log_level: LogLevel::Error,
    };
    pub const UNAUTHORIZED: Self = Self {
        value: "unauthorized",
        http_status: Status::Unauthorized,
        log_level: LogLevel::Info,
    };
    pub const FORBIDDEN: Self = Self {
        value: "forbidden",
        http_status: Status::Forbidden,
        log_level: LogLevel::Warn,
    };
    pub const DATABASE_ERROR: Self = Self {
        value: "database_error",
        http_status: Status::InternalServerError,
        log_level: LogLevel::Error,
    };
    pub const BAD_REQUEST: Self = Self {
        value: "bad_request",
        http_status: Status::BadRequest,
        log_level: LogLevel::Info,
    };
    pub const NOT_FOUND: Self = Self {
        value: "not_found",
        http_status: Status::NotFound,
        log_level: LogLevel::Info,
    };
    pub fn unknown(status: Status) -> Self {
        Self {
            value: "unknown",
            http_status: status,
            log_level: if (500..600).contains(&status.code) {
                // Server error
                LogLevel::Error
            } else {
                // Client error
                LogLevel::Warn
            },
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.value, f)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    code: &'static str,
    message: String,
    /// HTTP status to return for this error.
    pub http_status: Status,
    http_headers: Vec<(String, String)>,
    log_level: LogLevel,
    /// Whether or not the error has already been logged.
    /// This way we can make sure an error is not logged twice.
    logged: AtomicBool,
}

impl Error {
    pub fn new(code: ErrorCode, message: String, http_headers: Vec<(String, String)>) -> Self {
        Self {
            code: code.value,
            message,
            http_status: code.http_status,
            http_headers,
            log_level: code.log_level,
            logged: AtomicBool::new(false),
        }
    }
}

impl Error {
    /// Log the error.
    fn log(&self) {
        if self.logged.load(Ordering::Relaxed) {
            return;
        }

        // NOTE: `tracing` does not allow passing the log level dynamically
        //   therefore we introduced this custom `LogLevel` type and do a manual mapping.
        match self.log_level {
            LogLevel::Trace => trace!("{}", self.message),
            LogLevel::Debug => debug!("{}", self.message),
            LogLevel::Info => info!("{}", self.message),
            LogLevel::Warn => warn!("{}", self.message),
            LogLevel::Error => error!("{}", self.message),
        }

        self.logged.store(true, Ordering::Relaxed);
    }

    fn add_headers(&self, response: &mut Response<'_>) {
        for (name, value) in self.http_headers.iter() {
            response.set_header(Header::new(name.clone(), value.clone()));
        }
    }

    fn as_json(&self) -> String {
        json!({
            "reason": self.code,
        })
        .to_string()
    }

    /// Construct the HTTP response.
    fn as_response(&self) -> response::Result<'static> {
        let body = self.as_json();
        let mut response = Response::build()
            .status(self.http_status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()?;

        self.add_headers(&mut response);

        Ok(response)
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        self.log();
        self.as_response()
    }
}

pub trait HttpApiError: std::fmt::Display {
    fn code(&self) -> ErrorCode;
    fn message(&self) -> String {
        format!("{self}")
    }
    fn http_headers(&self) -> Vec<(String, String)> {
        vec![]
    }
}

impl<E: HttpApiError> From<E> for Error {
    fn from(error: E) -> Self {
        Self::new(error.code(), error.message(), error.http_headers())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Feature not implemented yet: {0}")]
pub struct NotImplemented(pub &'static str);
impl HttpApiError for NotImplemented {
    fn code(&self) -> ErrorCode {
        ErrorCode::NOT_IMPLEMENTED
    }
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

#[derive(Debug, thiserror::Error)]
#[error("Forbidden: {0}")]
pub struct Forbidden(pub String);
impl HttpApiError for Forbidden {
    fn code(&self) -> ErrorCode {
        ErrorCode::FORBIDDEN
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unknown database error")]
pub struct UnknownDbErr;
impl HttpApiError for UnknownDbErr {
    fn code(&self) -> ErrorCode {
        ErrorCode::DATABASE_ERROR
    }
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

/// HTTP status (used by the [default catcher](https://rocket.rs/guide/v0.5/requests/#default-catchers)
/// to change the output format).
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HTTPStatus(pub Status);
impl HttpApiError for HTTPStatus {
    fn code(&self) -> ErrorCode {
        ErrorCode::unknown(self.0)
    }
}

pub trait CustomErrorCode {
    fn error_code(&self) -> ErrorCode;
}

#[macro_export]
macro_rules! impl_into_error {
    ($t:ty) => {
        impl HttpApiError for $t {
            fn code(&self) -> ErrorCode {
                CustomErrorCode::error_code(self)
            }
            fn message(&self) -> String {
                format!("{} error: {self}", stringify!($t))
            }
        }
    };
    ($t:ty, $code:expr) => {
        impl HttpApiError for $t {
            fn code(&self) -> ErrorCode {
                $code
            }
            fn message(&self) -> String {
                format!("{} error: {self}", stringify!($t))
            }
        }
    };
    ($t:ty, $code:expr, $headers:expr) => {
        impl HttpApiError for $t {
            fn code(&self) -> ErrorCode {
                $code
            }
            fn message(&self) -> String {
                format!("{} error: {self}", stringify!($t))
            }
            fn http_headers(&self) -> Vec<(String, String)> {
                $headers
            }
        }
    };
}

impl_into_error!(sea_orm::DbErr, ErrorCode::DATABASE_ERROR);

impl_into_error!(server_ctl::Error, ErrorCode::INTERNAL_SERVER_ERROR);

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

impl CustomErrorCode for jwt_service::Error {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::Sign(_) | Self::Other(_) => ErrorCode::INTERNAL_SERVER_ERROR,
            Self::Verify(_) | Self::InvalidClaim(_) => ErrorCode::UNAUTHORIZED,
        }
    }
}
impl_into_error!(jwt_service::Error);

impl CustomErrorCode for auth_service::Error {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvalidCredentials => ErrorCode::UNAUTHORIZED,
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(auth_service::Error);

impl CustomErrorCode for user_service::Error {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateUser(err) => err.code(),
        }
    }
}
impl_into_error!(user_service::Error);

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
            Self::CouldNotLogIn(_) | Self::InvalidJwt(_) | Self::MissingProsodyToken(_) => {
                ErrorCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
impl_into_error!(CreateServiceAccountError);

impl CustomErrorCode for std::io::Error {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::INTERNAL_SERVER_ERROR
    }
}
impl_into_error!(std::io::Error);
