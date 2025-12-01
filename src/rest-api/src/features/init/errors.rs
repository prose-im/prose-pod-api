// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::init::errors::*;

use crate::error::prelude::*;

impl HttpApiError for FirstAccountAlreadyCreated {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "first_account_already_created",
            http_status: StatusCode::CONFLICT,
            log_level: LogLevel::Info,
        }
    }
}
