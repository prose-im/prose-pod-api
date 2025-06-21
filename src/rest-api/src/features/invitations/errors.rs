// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    app_config::MissingConfiguration,
    invitations::{
        invitation_controller::InvitationNotFound, CannotAcceptInvitation, InvitationAcceptError,
        InvitationExpired, InvitationResendError, InviteMemberError, NoInvitationForToken,
    },
    notifications::notifier::email::EmailNotificationCreateError,
};

use crate::error::prelude::*;

impl ErrorCode {
    const INVITATION_NOT_FOUND: Self = Self {
        value: "invitation_not_found",
        http_status: StatusCode::NOT_FOUND,
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

impl HttpApiError for InvitationAcceptError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateUser(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for CannotAcceptInvitation {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(err) => err.code(),
            Self::InvitationExpired(err) => err.code(),
            Self::MemberAlreadyExists => ErrorCode::MEMBER_ALREADY_EXISTS,
            Self::AcceptError(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for NoInvitationForToken {
    fn code(&self) -> ErrorCode {
        ErrorCode::INVITATION_NOT_FOUND
    }
}

impl HttpApiError for InvitationExpired {
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

impl HttpApiError for InvitationResendError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(_) => ErrorCode::INVITATION_NOT_FOUND,
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for InviteMemberError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationConfict => ErrorCode::INVITATION_ALREADY_EXISTS,
            Self::UsernameConfict => ErrorCode::MEMBER_ALREADY_EXISTS,
            #[cfg(debug_assertions)]
            Self::CouldNotAutoAcceptInvitation(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
}
