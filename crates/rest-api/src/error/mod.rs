// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;

use std::{
    str::FromStr as _,
    sync::atomic::{AtomicBool, Ordering},
};

use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use tracing::*;

pub use self::errors::*;

pub mod prelude {
    pub use axum::http::StatusCode;

    pub use crate::{error, impl_into_error};

    pub use super::{CustomErrorCode, Error, ErrorCode, HttpApiError, LogLevel};
}

#[derive(Debug)]
pub struct ErrorCode {
    /// User-facing error code (a string for easier understanding).
    pub value: &'static str,
    /// HTTP status to return for this error.
    pub http_status: StatusCode,
    pub log_level: LogLevel,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.value, f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{message}")]
pub struct Error {
    #[serde(rename = "error")]
    code: &'static str,

    message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    debug_info: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    recovery_suggestions: Vec<String>,

    /// HTTP status to return for this error.
    #[serde(skip_serializing)]
    pub http_status: StatusCode,

    #[serde(skip_serializing)]
    http_headers: Vec<(String, String)>,

    #[serde(skip_serializing)]
    log_level: LogLevel,

    /// Whether or not the error has already been logged.
    /// This way we can make sure an error is not logged twice.
    #[serde(skip_serializing)]
    logged: AtomicBool,
}

impl Error {
    pub fn new(
        code: ErrorCode,
        message: String,
        debug_info: Option<serde_json::Value>,
        recovery_suggestions: Vec<String>,
        http_headers: Vec<(String, String)>,
    ) -> Self {
        Self {
            code: code.value,
            message,
            debug_info,
            recovery_suggestions,
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

        let message = match self.recovery_suggestions.len() {
            0 => self.message.clone(),
            1 => format!(
                "{} (recovery suggestion: {})",
                self.message, self.recovery_suggestions[0],
            ),
            _ => format!(
                "{} (recovery suggestions: {:?})",
                self.message, self.recovery_suggestions,
            ),
        };

        // NOTE: `tracing` does not allow passing the log level dynamically
        //   therefore we introduced this custom `LogLevel` type and do a manual mapping.
        match self.log_level {
            LogLevel::Trace => trace!("{message}"),
            LogLevel::Debug => debug!("{message}"),
            LogLevel::Info => info!("{message}"),
            LogLevel::Warn => warn!("{message}"),
            LogLevel::Error => error!("{message}"),
        };

        self.logged.store(true, Ordering::Relaxed);
    }

    fn add_headers(&self, headers: &mut HeaderMap) {
        for (name, value) in self.http_headers.iter() {
            // FIXME: Store typed values in `http_headers`.
            headers.insert(
                HeaderName::from_str(&name).unwrap(),
                HeaderValue::from_str(&value).unwrap(),
            );
        }
    }

    fn as_json(&self) -> serde_json::Value {
        if cfg!(debug_assertions) {
            serde_json::to_value(self).unwrap_or_else(|_| {
                json!({
                    "error": self.code,
                    "message": self.message,
                    "debug_info": self.debug_info,
                })
            })
        } else {
            json!({
                "error": self.code,
            })
        }
    }

    /// Construct the HTTP response.
    fn as_response(&self) -> Response {
        let mut builder = Response::builder()
            .status(self.http_status)
            .header(CONTENT_TYPE, "application/json");

        self.add_headers(builder.headers_mut().unwrap());

        let body = axum::body::Body::from(self.as_json().to_string());
        builder.body(body).unwrap()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.log();
        self.as_response()
    }
}

pub trait HttpApiError: std::fmt::Display {
    fn code(&self) -> ErrorCode;
    fn message(&self) -> String {
        format!("{self}")
    }
    fn debug_info(&self) -> Option<serde_json::Value> {
        None
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![]
    }
    fn http_headers(&self) -> Vec<(String, String)> {
        vec![]
    }
}

impl<E: HttpApiError> From<E> for Error {
    fn from(error: E) -> Self {
        Self::new(
            error.code(),
            error.message(),
            error.debug_info(),
            error.recovery_suggestions(),
            error.http_headers(),
        )
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
