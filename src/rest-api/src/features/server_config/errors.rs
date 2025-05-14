// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::errors::*;

use crate::{error::prelude::*, features::server_config::SERVER_CONFIG_ROUTE};

impl ErrorCode {
    pub const SERVER_CONFIG_NOT_INITIALIZED: Self = Self {
        value: "server_config_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}

impl HttpApiError for ServerConfigAlreadyInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "server_config_already_initialized",
            http_status: StatusCode::CONFLICT,
            log_level: LogLevel::Info,
        }
    }
}

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
