// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    errors::MissingConfiguration,
    invitations::errors::{
        InvitationAlreadyExists, InvitationNotFound, InvitationNotFoundForToken,
        MemberAlreadyExists, UsernameAlreadyTaken,
    },
    notifications::notifier::email::EmailNotificationCreateError,
};

use crate::error::prelude::*;

impl ErrorCode {
    const INVITATION_NOT_FOUND: Self = Self {
        value: "invitation_not_found",
        http_status: StatusCode::GONE,
        log_level: LogLevel::Info,
    };
    const INVITATION_ALREADY_EXISTS: Self = Self {
        value: "invitation_already_exists",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
    const MEMBER_ALREADY_EXISTS: Self = Self {
        value: "member_already_exists",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}

impl HttpApiError for InvitationNotFound {
    fn code(&self) -> ErrorCode {
        ErrorCode::INVITATION_NOT_FOUND
    }
}

impl HttpApiError for MemberAlreadyExists {
    fn code(&self) -> ErrorCode {
        ErrorCode::MEMBER_ALREADY_EXISTS
    }
}

impl HttpApiError for InvitationNotFoundForToken {
    fn code(&self) -> ErrorCode {
        ErrorCode::INVITATION_NOT_FOUND
    }
}

impl HttpApiError for EmailNotificationCreateError {
    fn code(&self) -> ErrorCode {
        match self {
            EmailNotificationCreateError::AppConfig(MissingConfiguration(_)) => {
                ErrorCode::MISSING_CONFIG
            }
            EmailNotificationCreateError::ParseTo(_) => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpApiError for InvitationAlreadyExists {
    fn code(&self) -> ErrorCode {
        ErrorCode::INVITATION_ALREADY_EXISTS
    }
}

impl HttpApiError for UsernameAlreadyTaken {
    fn code(&self) -> ErrorCode {
        ErrorCode::MEMBER_ALREADY_EXISTS
    }
}
