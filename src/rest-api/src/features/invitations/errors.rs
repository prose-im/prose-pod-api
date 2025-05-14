// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    app_config::MissingConfiguration,
    invitations::{
        invitation_controller::InvitationNotFound, CannotAcceptInvitation, InvitationAcceptError,
        InvitationResendError, InviteMemberError, SendWorkspaceInvitationError,
    },
    notifications::notifier::email::EmailNotificationCreateError,
    pod_config::PodConfigField,
};

use crate::error::prelude::*;

impl ErrorCode {
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
        ErrorCode {
            value: "invitation_not_found",
            http_status: StatusCode::NOT_FOUND,
            log_level: LogLevel::Info,
        }
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
            Self::InvitationNotFound => ErrorCode::UNAUTHORIZED,
            Self::ExpiredAcceptToken => ErrorCode::NOT_FOUND,
            Self::MemberAlreadyExists => ErrorCode::MEMBER_ALREADY_EXISTS,
            Self::AcceptError(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
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

impl HttpApiError for SendWorkspaceInvitationError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateEmailNotification(err) => err.code(),
            Self::NotificationService(err) => err.code(),
        }
    }
}

impl HttpApiError for InvitationResendError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(_) => ErrorCode::NOT_FOUND,
            Self::CouldNotSendInvitation(err) => err.code(),
            Self::PodConfigMissing(PodConfigField::PodAddress) => {
                ErrorCode::POD_ADDRESS_NOT_INITIALIZED
            }
            Self::PodConfigMissing(PodConfigField::DashboardUrl) => {
                ErrorCode::DASHBOARD_URL_NOT_INITIALIZED
            }
            Self::Internal(err) => err.code(),
        }
    }
}

impl HttpApiError for InviteMemberError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidJid(_) => ErrorCode::BAD_REQUEST,
            Self::InvitationConfict => ErrorCode::INVITATION_ALREADY_EXISTS,
            Self::UsernameConfict => ErrorCode::MEMBER_ALREADY_EXISTS,
            Self::PodConfigMissing(PodConfigField::PodAddress) => {
                ErrorCode::POD_ADDRESS_NOT_INITIALIZED
            }
            Self::PodConfigMissing(PodConfigField::DashboardUrl) => {
                ErrorCode::DASHBOARD_URL_NOT_INITIALIZED
            }
            #[cfg(debug_assertions)]
            Self::CouldNotAutoAcceptInvitation(err) => err.code(),
            Self::Internal(err) => err.code(),
        }
    }
}
