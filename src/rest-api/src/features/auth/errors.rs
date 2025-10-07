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

impl HttpApiError for InvalidAuthToken {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "invalid_auth_token",
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

impl HttpApiError for MissingEmailAddress {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "missing_email_address",
            http_status: StatusCode::PRECONDITION_FAILED,
            log_level: LogLevel::Warn,
        }
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "An admin needs to set the email address of {jid}.",
            jid = self.0.to_string(),
        )]
    }
}

impl HttpApiError for CannotResetPassword {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "cannot_reset_password",
            http_status: StatusCode::FORBIDDEN,
            log_level: LogLevel::Info,
        }
    }
}

impl HttpApiError for PasswordResetTokenExpired {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "password_reset_token_expired",
            http_status: StatusCode::NOT_FOUND,
            log_level: LogLevel::Info,
        }
    }
}
