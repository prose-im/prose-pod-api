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
use serde_json::json;
use serdev::Serialize;
use tracing::*;

pub use self::errors::*;

pub mod prelude {
    pub use axum::http::StatusCode;

    pub use crate::{error, impl_into_error};

    pub use super::{CustomErrorCode, Error, ErrorCode, HttpApiError, LogLevel};
}

pub(crate) static DETAILED_ERROR_REPONSES: AtomicBool = AtomicBool::new(false);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, thiserror::Error)]
#[derive(Serialize)]
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
        }
    }
}

impl Error {
    /// Log the error.
    fn log(&self) {
        let mut messages = Vec::<String>::with_capacity(3);

        messages.push(self.message.clone());

        match self.recovery_suggestions.len() {
            0 => {}
            1 => messages.push(format!(
                "Recovery suggestion: {}",
                self.recovery_suggestions[0],
            )),
            _ => messages.push(format!(
                "Recovery suggestions: {:?}",
                self.recovery_suggestions,
            )),
        };

        if let Some(ref debug_info) = self.debug_info {
            messages.push(format!("Debug info: {debug_info}"));
        }

        let log_line = messages.join("\n\n");

        // NOTE: `tracing` does not allow passing the log level dynamically
        //   therefore we introduced this custom `LogLevel` type and do a manual mapping.
        match self.log_level {
            LogLevel::Trace => trace!("{log_line}"),
            LogLevel::Debug => debug!("{log_line}"),
            LogLevel::Info => info!("{log_line}"),
            LogLevel::Warn => warn!("{log_line}"),
            LogLevel::Error => error!("{log_line}"),
        };
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
        if DETAILED_ERROR_REPONSES.load(Ordering::Relaxed) {
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

impl From<std::convert::Infallible> for Error {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}

pub trait CustomErrorCode {
    fn error_code(&self) -> ErrorCode;
}

#[macro_export]
macro_rules! impl_into_error {
    ($t:ty) => {
        impl crate::error::HttpApiError for $t {
            fn code(&self) -> ErrorCode {
                CustomErrorCode::error_code(self)
            }
            fn message(&self) -> String {
                format!("{} error: {self}", stringify!($t))
            }
        }
    };
    (
        $t:ty
        , code: $code:literal
        , http_status: $status:ident
        , log_level: $log_level:ident
        $(,)?
    ) => {
        impl crate::error::HttpApiError for $t {
            fn code(&self) -> crate::error::ErrorCode {
                crate::error::ErrorCode {
                    value: $code,
                    http_status: axum::http::StatusCode::$status,
                    log_level: crate::error::LogLevel::$log_level,
                }
            }
        }
    };
    ($t:ty, $code:expr) => {
        impl crate::error::HttpApiError for $t {
            fn code(&self) -> ErrorCode {
                $code
            }
            fn message(&self) -> String {
                format!("{} error: {self}", stringify!($t))
            }
        }
    };
    ($t:ty, $code:expr, $headers:expr) => {
        impl crate::error::HttpApiError for $t {
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
