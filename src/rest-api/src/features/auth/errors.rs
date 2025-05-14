// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::auth::errors::*;

use crate::error::prelude::*;

impl HttpApiError for InvalidCredentials {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "invalid_credentials",
            http_status: StatusCode::UNAUTHORIZED,
            log_level: LogLevel::Info,
        }
    }
}

impl HttpApiError for CannotChangeOwnRole {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "cannot_change_own_role",
            http_status: StatusCode::FORBIDDEN,
            log_level: LogLevel::Info,
        }
    }
}

impl HttpApiError for CannotAssignRole {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "cannot_assign_role",
            http_status: StatusCode::FORBIDDEN,
            log_level: LogLevel::Info,
        }
    }
}
