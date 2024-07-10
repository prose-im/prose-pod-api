// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::io::Cursor;

use http_auth_basic::AuthBasicError;
use rocket::http::{ContentType, Header, Status};
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde_json::json;
use service::controllers::init_controller::{
    InitFirstAccountError, InitServerConfigError, InitWorkspaceError,
};
use service::controllers::invitation_controller::{
    InvitationAcceptError, InvitationCancelError, InvitationRejectError, InvitationResendError,
    InviteMemberError,
};
#[cfg(debug_assertions)]
use service::services::jwt_service;
use service::services::{
    auth_service, invitation_service, notifier, server_ctl, server_manager,
    user_service::{self, UserCreateError},
    xmpp_service,
};
use service::{sea_orm, MutationError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Feature not implemented yet: {0}")]
    NotImplemented(&'static str),
    /// Internal server error.
    /// Use it only when a nearly-impossible code path is taken.
    #[error("Internal server error: {reason}")]
    InternalServerError { reason: String },
    #[error("Unauthorized.")]
    Unauthorized,
    #[error("Unknown database error.")]
    UnknownDbErr,
    #[error("Database error: {0}")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Workspace not initialized. Call `PUT {}` to initialize it.", uri!(crate::v1::init::init_workspace))]
    WorkspaceNotInitialized,
    #[error("Workspace already initialized.")]
    WorkspaceAlreadyInitialized,
    #[error("XMPP server not initialized. Call `PUT {}` to initialize it.", uri!(crate::v1::init::init_server_config))]
    ServerConfigNotInitialized,
    #[error("XMPP server already initialized.")]
    ServerConfigAlreadyInitialized,
    #[error("First XMPP account already created.")]
    FirstAccountAlreadyCreated,
    #[error("ServerCtl error: {0}")]
    ServerCtlErr(#[from] server_ctl::Error),
    #[error("XmppService error: {0}")]
    XmppServiceErr(#[from] xmpp_service::Error),
    #[error("Bad request: {reason}")]
    BadRequest { reason: String },
    #[error("Mutation error: {0}")]
    MutationErr(#[from] MutationError),
    #[error("Not found: {reason}")]
    NotFound { reason: String },
    #[error("Notifier error: {0}")]
    NotifierError(#[from] notifier::Error),
    #[error("Basic auth error: {0}")]
    BasicAuthError(AuthBasicError),
    /// HTTP status (used by the [default catcher](https://rocket.rs/guide/v0.5/requests/#default-catchers)
    /// to change the output format).
    #[error("{0}")]
    HTTPStatus(Status),
}

impl Error {
    /// Log the error.
    pub fn log(&self) {
        if (500..600).contains(&self.http_status().code) {
            // Server error
            error!("{self}");
        } else {
            // Client error
            warn!("{self}");
        }
    }

    /// HTTP status to return for this error.
    pub fn http_status(&self) -> Status {
        match self {
            Self::NotImplemented(_) => Status::NotImplemented,
            Self::InternalServerError { .. } => Status::InternalServerError,
            Self::Unauthorized => Status::Unauthorized,
            Self::UnknownDbErr => Status::InternalServerError,
            Self::DbErr(_) => Status::InternalServerError,
            Self::WorkspaceNotInitialized | Self::ServerConfigNotInitialized => Status::BadRequest,
            Self::WorkspaceAlreadyInitialized
            | Self::ServerConfigAlreadyInitialized
            | Self::FirstAccountAlreadyCreated => Status::Conflict,
            Self::ServerCtlErr(_) | Self::XmppServiceErr(_) => Status::InternalServerError,
            Self::BadRequest { .. } => Status::BadRequest,
            Self::MutationErr(_) => Status::InternalServerError,
            Self::NotFound { .. } => Status::NotFound,
            Self::NotifierError(_) => Status::InternalServerError,
            Self::BasicAuthError(_) => Status::Unauthorized,
            Self::HTTPStatus(s) => s.to_owned(),
        }
    }

    /// User-facing error code (a string for easier understanding).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotImplemented(_) => "not_implemented",
            Self::InternalServerError { .. } => "internal_server_error",
            Self::Unauthorized => "unauthorized",
            Self::UnknownDbErr => "database_error",
            Self::DbErr(_) => "database_error",
            Self::WorkspaceNotInitialized => "workspace_not_initialized",
            Self::WorkspaceAlreadyInitialized => "workspace_already_initialized",
            Self::ServerConfigNotInitialized => "server_config_not_initialized",
            Self::ServerConfigAlreadyInitialized => "server_config_already_initialized",
            Self::FirstAccountAlreadyCreated => "first_account_already_created",
            Self::ServerCtlErr(_) | Self::XmppServiceErr(_) => "internal_server_error",
            Self::BadRequest { .. } => "bad_request",
            Self::MutationErr(MutationError::DbErr(_)) => "database_error",
            Self::MutationErr(MutationError::EntityNotFound { .. }) => "not_found",
            Self::NotFound { .. } => "not_found",
            Self::NotifierError(_) => "internal_server_error",
            Self::BasicAuthError(_) => "unauthorized",
            Self::HTTPStatus(_) => "unknown",
        }
    }

    pub fn add_headers(&self, response: &mut Response<'_>) {
        match self {
            Self::Unauthorized => {
                response.set_header(Header::new(
                    "WWW-Authenticate",
                    r#"Bearer realm="Admin only area", charset="UTF-8""#,
                ));
            }
            Self::BasicAuthError(_) => {
                response.set_header(Header::new(
                    "WWW-Authenticate",
                    r#"Basic realm="Admin only area", charset="UTF-8""#,
                ));
            }
            _ => {}
        }
    }

    pub(crate) fn as_json(&self) -> String {
        json!({
            "reason": self.code(),
        })
        .to_string()
    }

    /// Construct the HTTP response.
    pub(crate) fn as_response(&self) -> response::Result<'static> {
        let body = self.as_json();
        let mut response = Response::build()
            .status(self.http_status())
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()?;

        self.add_headers(&mut response);

        Ok(response)
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        self.log();
        self.as_response()
    }
}

impl From<AuthBasicError> for Error {
    fn from(value: AuthBasicError) -> Self {
        Self::BasicAuthError(value)
    }
}

impl From<auth_service::Error> for Error {
    fn from(value: auth_service::Error) -> Self {
        match value {
            auth_service::Error::InvalidCredentials => Self::Unauthorized,
            e => Self::InternalServerError {
                reason: format!("Auth error: {e}"),
            },
        }
    }
}

#[cfg(debug_assertions)]
impl From<jwt_service::Error> for Error {
    fn from(value: jwt_service::Error) -> Self {
        Self::InternalServerError {
            reason: format!("JWT error: {value}"),
        }
    }
}

impl From<user_service::Error> for Error {
    fn from(value: user_service::Error) -> Self {
        Self::InternalServerError {
            reason: value.to_string(),
        }
    }
}

impl From<UserCreateError> for Error {
    fn from(value: UserCreateError) -> Self {
        user_service::Error::from(value).into()
    }
}

impl From<invitation_service::Error> for Error {
    fn from(value: invitation_service::Error) -> Self {
        Self::InternalServerError {
            reason: value.to_string(),
        }
    }
}

impl From<invitation_service::InvitationAcceptError> for Error {
    fn from(value: invitation_service::InvitationAcceptError) -> Self {
        invitation_service::Error::from(value).into()
    }
}

impl From<server_manager::Error> for Error {
    fn from(value: server_manager::Error) -> Self {
        match value {
            server_manager::Error::ServerConfigAlreadyInitialized => {
                Self::ServerConfigAlreadyInitialized
            }
            server_manager::Error::ServerCtl(err) => Self::ServerCtlErr(err),
            server_manager::Error::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InitServerConfigError> for Error {
    fn from(value: InitServerConfigError) -> Self {
        match value {
            InitServerConfigError::CouldNotInitServerConfig(err) => Self::from(err),
        }
    }
}

impl From<InitWorkspaceError> for Error {
    fn from(value: InitWorkspaceError) -> Self {
        match value {
            InitWorkspaceError::WorkspaceAlreadyInitialized => Self::WorkspaceAlreadyInitialized,
            InitWorkspaceError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InitFirstAccountError> for Error {
    fn from(value: InitFirstAccountError) -> Self {
        match value {
            InitFirstAccountError::FirstAccountAlreadyCreated => Self::FirstAccountAlreadyCreated,
            InitFirstAccountError::InvalidJid(_) => Self::BadRequest {
                reason: value.to_string(),
            },
            InitFirstAccountError::CouldNotCreateFirstAccount(_) => Self::InternalServerError {
                reason: value.to_string(),
            },
            InitFirstAccountError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InviteMemberError> for Error {
    fn from(value: InviteMemberError) -> Self {
        match value {
            InviteMemberError::InvalidJid(_) => Self::BadRequest {
                reason: value.to_string(),
            },
            InviteMemberError::CouldNotUpdateInvitationStatus { .. }
            | InviteMemberError::CouldNotAutoAcceptInvitation(_) => Self::InternalServerError {
                reason: value.to_string(),
            },
            InviteMemberError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InvitationAcceptError> for Error {
    fn from(value: InvitationAcceptError) -> Self {
        match value {
            InvitationAcceptError::InvitationNotFound => Self::Unauthorized,
            InvitationAcceptError::ExpiredAcceptToken => Self::NotFound {
                reason: value.to_string(),
            },
            InvitationAcceptError::ServiceError(err) => Self::from(err),
            InvitationAcceptError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InvitationRejectError> for Error {
    fn from(value: InvitationRejectError) -> Self {
        match value {
            InvitationRejectError::InvitationNotFound => Self::Unauthorized,
            InvitationRejectError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InvitationResendError> for Error {
    fn from(value: InvitationResendError) -> Self {
        match value {
            InvitationResendError::InvitationNotFound(_) => Self::NotFound {
                reason: value.to_string(),
            },
            InvitationResendError::CouldNotSendInvitation(_) => Self::InternalServerError {
                reason: value.to_string(),
            },
            InvitationResendError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<InvitationCancelError> for Error {
    fn from(value: InvitationCancelError) -> Self {
        match value {
            InvitationCancelError::DbErr(err) => Self::DbErr(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::InternalServerError {
            reason: format!("IO error: {value}"),
        }
    }
}
