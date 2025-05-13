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
