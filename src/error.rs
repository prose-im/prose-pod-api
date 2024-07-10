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

#[derive(Debug)]
enum ErrorCode {
    NotImplemented,
    InternalServerError,
    Unauthorized,
    DatabaseError,
    WorkspaceNotInitialized,
    WorkspaceAlreadyInitialized,
    ServerConfigNotInitialized,
    ServerConfigAlreadyInitialized,
    FirstAccountAlreadyCreated,
    BadRequest,
    NotFound,
    Unknown(Status),
}

impl ErrorCode {
    /// User-facing error code (a string for easier understanding).
    fn value(&self) -> &'static str {
        match self {
            Self::NotImplemented => "not_implemented",
            Self::InternalServerError => "internal_server_error",
            Self::Unauthorized => "unauthorized",
            Self::DatabaseError => "database_error",
            Self::WorkspaceNotInitialized => "workspace_not_initialized",
            Self::WorkspaceAlreadyInitialized => "workspace_already_initialized",
            Self::ServerConfigNotInitialized => "server_config_not_initialized",
            Self::ServerConfigAlreadyInitialized => "server_config_already_initialized",
            Self::FirstAccountAlreadyCreated => "first_account_already_created",
            Self::BadRequest => "bad_request",
            Self::NotFound => "not_found",
            Self::Unknown(_) => "unknown",
        }
    }

    /// HTTP status to return for this error.
    pub fn http_status(&self) -> Status {
        match self {
            Self::NotImplemented => Status::NotImplemented,
            Self::InternalServerError | Self::DatabaseError => Status::InternalServerError,
            Self::Unauthorized => Status::Unauthorized,
            Self::BadRequest | Self::WorkspaceNotInitialized | Self::ServerConfigNotInitialized => {
                Status::BadRequest
            }
            Self::WorkspaceAlreadyInitialized
            | Self::ServerConfigAlreadyInitialized
            | Self::FirstAccountAlreadyCreated => Status::Conflict,
            Self::NotFound => Status::NotFound,
            Self::Unknown(s) => s.to_owned(),
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Feature not implemented yet: {0}")]
    NotImplemented(&'static str),
    /// Internal server error.
    /// Use it only when a nearly-impossible code path is taken.
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Database error: {0}")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Unknown database error")]
    UnknownDbErr,
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
    // #[error("{code}: {message}")]
    // Custom { code: ErrorCode, message: String },
    #[error("JWT error: {0}")]
    JwtError(#[from] jwt_service::Error),
    #[error("Auth error: {0}")]
    AuthError(#[from] auth_service::Error),
    #[error("User creation error: {0}")]
    UserCreateError(#[from] UserCreateError),
    #[error("User service error: {0}")]
    UserServiceError(#[from] user_service::Error),
    #[error("Server manager error: {0}")]
    ServerManagerError(#[from] server_manager::Error),
    #[error("Server config init error: {0}")]
    InitServerConfigError(#[from] InitServerConfigError),
    #[error("Workspace init error: {0}")]
    InitWorkspaceError(#[from] InitWorkspaceError),
    #[error("First account init error: {0}")]
    InitFirstAccountError(#[from] InitFirstAccountError),
    #[error("Invite member error: {0}")]
    InviteMemberError(#[from] InviteMemberError),
    #[error("Invitation service accept error: {0}")]
    InvitationServiceAcceptError(#[from] invitation_service::InvitationAcceptError),
    #[error("Invitation accept error: {0}")]
    InvitationAcceptError(#[from] InvitationAcceptError),
    #[error("Invitation reject error: {0}")]
    InvitationRejectError(#[from] InvitationRejectError),
    #[error("Invitation resend error: {0}")]
    InvitationResendError(#[from] InvitationResendError),
    #[error("Invitation cancel error: {0}")]
    InvitationCancelError(#[from] InvitationCancelError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    /// Log the error.
    fn log(&self) {
        if (500..600).contains(&self.http_status().code) {
            // Server error
            error!("{self}");
        } else {
            // Client error
            warn!("{self}");
        }
    }

    /// HTTP status to return for this error.
    pub(crate) fn http_status(&self) -> Status {
        self.code().http_status()
    }

    /// User-facing error code (a string for easier understanding).
    fn code(&self) -> ErrorCode {
        match self {
            Self::NotImplemented(_) => ErrorCode::NotImplemented,
            Self::InternalServerError(_)
            | Self::ServerCtlErr(_)
            | Self::XmppServiceErr(_)
            | Self::NotifierError(_) => ErrorCode::InternalServerError,
            Self::Unauthorized(_) | Self::BasicAuthError(_) => ErrorCode::Unauthorized,
            Self::DbErr(_) | Self::UnknownDbErr => ErrorCode::DatabaseError,
            Self::WorkspaceNotInitialized => ErrorCode::WorkspaceNotInitialized,
            Self::WorkspaceAlreadyInitialized => ErrorCode::WorkspaceAlreadyInitialized,
            Self::ServerConfigNotInitialized => ErrorCode::ServerConfigNotInitialized,
            Self::ServerConfigAlreadyInitialized => ErrorCode::ServerConfigAlreadyInitialized,
            Self::FirstAccountAlreadyCreated => ErrorCode::FirstAccountAlreadyCreated,
            Self::BadRequest { .. } => ErrorCode::BadRequest,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::HTTPStatus(s) => ErrorCode::Unknown(s.to_owned()),

            Self::JwtError(jwt_service::Error::Sign(_) | jwt_service::Error::Other(_)) => {
                ErrorCode::InternalServerError
            }
            Self::JwtError(jwt_service::Error::Verify(_) | jwt_service::Error::InvalidClaim(_)) => {
                ErrorCode::Unauthorized
            }

            Self::AuthError(auth_service::Error::InvalidCredentials) => ErrorCode::Unauthorized,
            Self::AuthError(_) => ErrorCode::InternalServerError,

            Self::MutationErr(MutationError::DbErr(_)) => ErrorCode::DatabaseError,
            Self::MutationErr(MutationError::EntityNotFound { .. }) => ErrorCode::NotFound,

            Self::UserCreateError(UserCreateError::DbErr(_)) => ErrorCode::DatabaseError,
            Self::UserCreateError(_) => ErrorCode::InternalServerError,

            Self::UserServiceError(user_service::Error::CouldNotCreateUser(err)) => {
                Self::from(err.to_owned()).code()
            }

            Self::ServerManagerError(server_manager::Error::DbErr(_)) => ErrorCode::DatabaseError,
            Self::ServerManagerError(server_manager::Error::ServerConfigAlreadyInitialized) => {
                ErrorCode::ServerConfigAlreadyInitialized
            }
            Self::ServerManagerError(server_manager::Error::ServerCtl(_)) => {
                ErrorCode::InternalServerError
            }

            Self::InitServerConfigError(InitServerConfigError::CouldNotInitServerConfig(err)) => {
                Self::from(err.to_owned()).code()
            }

            Self::InitWorkspaceError(InitWorkspaceError::WorkspaceAlreadyInitialized) => {
                ErrorCode::WorkspaceAlreadyInitialized
            }
            Self::InitWorkspaceError(InitWorkspaceError::DbErr(_)) => ErrorCode::DatabaseError,

            Self::InitFirstAccountError(InitFirstAccountError::FirstAccountAlreadyCreated) => {
                ErrorCode::FirstAccountAlreadyCreated
            }
            Self::InitFirstAccountError(InitFirstAccountError::InvalidJid(_)) => {
                ErrorCode::BadRequest
            }
            Self::InitFirstAccountError(InitFirstAccountError::CouldNotCreateFirstAccount(_)) => {
                ErrorCode::InternalServerError
            }
            Self::InitFirstAccountError(InitFirstAccountError::DbErr(_)) => {
                ErrorCode::DatabaseError
            }

            Self::InviteMemberError(InviteMemberError::InvalidJid(_)) => ErrorCode::BadRequest,
            Self::InviteMemberError(
                InviteMemberError::CouldNotUpdateInvitationStatus { .. }
                | InviteMemberError::CouldNotAutoAcceptInvitation(_),
            ) => ErrorCode::InternalServerError,
            Self::InviteMemberError(InviteMemberError::DbErr(_)) => ErrorCode::DatabaseError,

            Self::InvitationServiceAcceptError(
                invitation_service::InvitationAcceptError::DbErr(_),
            ) => ErrorCode::DatabaseError,
            Self::InvitationServiceAcceptError(_) => ErrorCode::InternalServerError,

            Self::InvitationAcceptError(InvitationAcceptError::InvitationNotFound) => {
                ErrorCode::Unauthorized
            }
            Self::InvitationAcceptError(InvitationAcceptError::ExpiredAcceptToken) => {
                ErrorCode::NotFound
            }
            Self::InvitationAcceptError(InvitationAcceptError::ServiceError(err)) => {
                Self::from(err.to_owned()).code()
            }
            Self::InvitationAcceptError(InvitationAcceptError::DbErr(_)) => {
                ErrorCode::DatabaseError
            }

            Self::InvitationRejectError(InvitationRejectError::InvitationNotFound) => {
                ErrorCode::Unauthorized
            }
            Self::InvitationRejectError(InvitationRejectError::DbErr(_)) => {
                ErrorCode::DatabaseError
            }

            Self::InvitationResendError(InvitationResendError::InvitationNotFound(_)) => {
                ErrorCode::NotFound
            }
            Self::InvitationResendError(InvitationResendError::CouldNotSendInvitation(_)) => {
                ErrorCode::InternalServerError
            }
            Self::InvitationResendError(InvitationResendError::DbErr(_)) => {
                ErrorCode::DatabaseError
            }

            Self::InvitationCancelError(InvitationCancelError::DbErr(_)) => {
                ErrorCode::DatabaseError
            }

            Self::IoError(_) => ErrorCode::InternalServerError,
        }
    }

    fn add_headers(&self, response: &mut Response<'_>) {
        match self {
            Self::Unauthorized(_) => {
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

    fn as_json(&self) -> String {
        json!({
            "reason": self.code().to_string(),
        })
        .to_string()
    }

    /// Construct the HTTP response.
    fn as_response(&self) -> response::Result<'static> {
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
