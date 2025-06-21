// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::init::InitFirstAccountError;

use crate::error::prelude::*;

impl ErrorCode {
    pub const FIRST_ACCOUNT_ALREADY_CREATED: Self = Self {
        value: "first_account_already_created",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}

impl HttpApiError for InitFirstAccountError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::FirstAccountAlreadyCreated => ErrorCode::FIRST_ACCOUNT_ALREADY_CREATED,
            Self::CouldNotCreateFirstAccount(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
