// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::workspace::errors::*;

use crate::error::prelude::*;

impl HttpApiError for WorkspaceAlreadyInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "workspace_already_initialized",
            http_status: StatusCode::CONFLICT,
            log_level: LogLevel::Info,
        }
    }
}
