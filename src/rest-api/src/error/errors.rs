// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::header::InvalidHeaderValue;
pub use service::errors::*;
use service::{sea_orm, xmpp::xmpp_service, MutationError};

use super::prelude::*;

impl HttpApiError for anyhow::Error {
    fn code(&self) -> ErrorCode {
        ErrorCode::INTERNAL_SERVER_ERROR
    }
    fn message(&self) -> String {
        format!("{self:#}")
    }
    fn debug_info(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(format!("{self:#?}")))
    }
}

macro_rules! impl_error_for_either {
    ($t:ident<$case1:ident$(, $cases:ident)+>) => {
        impl<$case1: HttpApiError$(, $cases: HttpApiError)+> HttpApiError for service::util::either::$t<$case1$(, $cases)+> {
            fn code(&self) -> ErrorCode {
                match self {
                    Self::$case1(err) => err.code(),
                    $(Self::$cases(err) => err.code(),)+
                }
            }
            fn message(&self) -> String {
                match self {
                    Self::$case1(err) => err.message(),
                    $(Self::$cases(err) => err.message(),)+
                }
            }
            fn debug_info(&self) -> Option<serde_json::Value> {
                match self {
                    Self::$case1(err) => err.debug_info(),
                    $(Self::$cases(err) => err.debug_info(),)+
                }
            }
            fn recovery_suggestions(&self) -> Vec<String> {
                match self {
                    Self::$case1(err) => err.recovery_suggestions(),
                    $(Self::$cases(err) => err.recovery_suggestions(),)+
                }
            }
            fn http_headers(&self) -> Vec<(String, String)> {
                match self {
                    Self::$case1(err) => err.http_headers(),
                    $(Self::$cases(err) => err.http_headers(),)+
                }
            }
        }
    };
}

impl_error_for_either!(Either<E1, E2>);
impl_error_for_either!(Either3<E1, E2, E3>);
impl_error_for_either!(Either4<E1, E2, E3, E4>);

impl ErrorCode {
    pub const NOT_IMPLEMENTED: Self = Self {
        value: "not_implemented",
        http_status: StatusCode::NOT_IMPLEMENTED,
        log_level: LogLevel::Error,
    };
}
#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
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
#[repr(transparent)]
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
    pub const PRECONDITION_REQUIRED: Self = Self {
        value: "precondition_required",
        http_status: StatusCode::PRECONDITION_REQUIRED,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Precondition required: {comment}")]
pub struct PreconditionRequired {
    pub comment: String,
}
impl HttpApiError for PreconditionRequired {
    fn code(&self) -> ErrorCode {
        ErrorCode::PRECONDITION_REQUIRED
    }
}

impl ErrorCode {
    pub const UNSUPPORTED_MEDIA_TYPE: Self = Self {
        value: "unsupported_media_type",
        http_status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Precondition required: {comment}")]
pub struct UnsupportedMediaType {
    pub comment: String,
}
impl HttpApiError for UnsupportedMediaType {
    fn code(&self) -> ErrorCode {
        ErrorCode::UNSUPPORTED_MEDIA_TYPE
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
    pub const MISSING_CONFIG: Self = Self {
        value: "missing_config",
        http_status: StatusCode::INTERNAL_SERVER_ERROR,
        log_level: LogLevel::Error,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Missing configuration: {config_id}")]
pub struct MissingConfig {
    /// Not a configuration key as it might change over time, but rather
    /// a stable configuration identifier that the Dashboard can use
    /// to redirect a user to the appropriate settings section.
    pub config_id: &'static str,
}
impl HttpApiError for MissingConfig {
    fn code(&self) -> ErrorCode {
        ErrorCode::MISSING_CONFIG
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
/// HTTP status (used to change the default output format).
#[derive(Debug, thiserror::Error)]
#[error("{status}")]
pub struct HTTPStatus {
    pub status: StatusCode,
    pub body: String,
}
impl HttpApiError for HTTPStatus {
    fn code(&self) -> ErrorCode {
        ErrorCode::unknown(self.status)
    }
    fn message(&self) -> String {
        self.body.clone()
    }
}

impl_into_error!(sea_orm::DbErr, ErrorCode::DATABASE_ERROR);

impl_into_error!(xmpp_service::Error, ErrorCode::INTERNAL_SERVER_ERROR);

impl CustomErrorCode for MutationError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::EntityNotFound { .. } => ErrorCode::NOT_FOUND,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(MutationError);

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

impl_into_error!(InvalidHeaderValue, ErrorCode::INTERNAL_SERVER_ERROR);

impl HttpApiError for service::errors::NotImplemented {
    fn code(&self) -> ErrorCode {
        ErrorCode::NOT_IMPLEMENTED
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct InvalidServerConfiguration(pub InvalidConfiguration);
