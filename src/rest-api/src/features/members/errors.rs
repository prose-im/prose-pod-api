// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::{member_controller::MemberNotFound, UserCreateError, UserDeleteError};

use crate::error::prelude::*;

impl HttpApiError for MemberNotFound {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "member_not_found",
            http_status: StatusCode::NOT_FOUND,
            log_level: LogLevel::Info,
        }
    }
}

impl CustomErrorCode for UserCreateError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::LimitReached => ErrorCode {
                value: "user_limit_reached",
                http_status: StatusCode::FORBIDDEN,
                log_level: LogLevel::Error,
            },
            Self::DbErr(err) => err.code(),
            Self::CouldNotCreateVCard(_)
            | Self::XmppServerCannotCreateUser(_)
            | Self::XmppServerCannotAddTeamMember(_)
            | Self::XmppServerCannotSetUserRole(_) => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(UserCreateError);

impl CustomErrorCode for UserDeleteError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CannotSelfRemove => ErrorCode {
                value: "cannot_remove_self",
                http_status: StatusCode::FORBIDDEN,
                log_level: LogLevel::Info,
            },
            Self::Forbidden(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
}
impl_into_error!(UserDeleteError);
