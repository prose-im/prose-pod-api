// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};

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
use service::controllers::workspace_controller::{
    WorkspaceControllerError, WorkspaceControllerInitError,
};
use service::services::server_manager::CreateServiceAccountError;
use service::services::{
    auth_service, invitation_service, jwt_service, notifier, server_ctl, server_manager,
    user_service::{self, UserCreateError},
    xmpp_service,
};
use service::{sea_orm, MutationError};

#[derive(Debug)]
pub(crate) enum ErrorCode {
    NotImplemented,
    InternalServerError,
    Unauthorized,
    Forbidden,
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
            Self::Forbidden => "forbidden",
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
            Self::Forbidden => Status::Forbidden,
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
        std::fmt::Display::fmt(self.value(), f)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    code: ErrorCode,
    message: String,
    headers: Vec<(String, String)>,
    /// Whether or not the error has already been logged.
    /// This way we can make sure an error is not logged twice.
    logged: AtomicBool,
}

impl Error {
    fn new(code: ErrorCode, message: String, headers: Vec<(String, String)>) -> Self {
        Self {
            code,
            message,
            headers,
            logged: AtomicBool::new(false),
        }
    }

    /// Log the error.
    fn log(&self) {
        if self.logged.load(Ordering::Relaxed) {
            return;
        }

        if (500..600).contains(&self.http_status().code) {
            // Server error
            error!("{}", self.message);
        } else {
            // Client error
            match self.code {
                ErrorCode::Forbidden | ErrorCode::Unknown(_) => warn!("{}", self.message),
                _ => info!("{}", self.message),
            }
        }

        self.logged.store(true, Ordering::Relaxed);
    }

    /// HTTP status to return for this error.
    pub(crate) fn http_status(&self) -> Status {
        self.code.http_status()
    }

    fn add_headers(&self, response: &mut Response<'_>) {
        for (name, value) in self.headers.iter() {
            response.set_header(Header::new(name.clone(), value.clone()));
        }
    }

    fn as_json(&self) -> String {
        json!({
            "reason": self.code.to_string(),
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

pub(crate) trait HttpApiError: std::fmt::Display {
    fn code(&self) -> ErrorCode;
    fn headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

macro_rules! impl_into_error_from_display {
    ($t:ty) => {
        impl From<$t> for Error {
            fn from(error: $t) -> Self {
                Self::new(error.code(), format!("{error}"), error.headers())
            }
        }
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Feature not implemented yet: {0}")]
pub struct NotImplemented(pub &'static str);
impl HttpApiError for NotImplemented {
    fn code(&self) -> ErrorCode {
        ErrorCode::NotImplemented
    }
}
impl_into_error_from_display!(NotImplemented);

/// Internal server error.
/// Use it only when a nearly-impossible code path is taken.
#[derive(Debug, thiserror::Error)]
#[error("Internal server error: {0}")]
pub struct InternalServerError(pub String);
impl HttpApiError for InternalServerError {
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}
impl_into_error_from_display!(InternalServerError);

#[derive(Debug, thiserror::Error)]
#[error("Unauthorized: {0}")]
pub struct Unauthorized(pub String);
impl HttpApiError for Unauthorized {
    fn code(&self) -> ErrorCode {
        ErrorCode::Unauthorized
    }
    fn headers(&self) -> Vec<(String, String)> {
        vec![(
            "WWW-Authenticate".into(),
            r#"Bearer realm="Admin only area", charset="UTF-8""#.into(),
        )]
    }
}
impl_into_error_from_display!(Unauthorized);

#[derive(Debug, thiserror::Error)]
#[error("Forbidden: {0}")]
pub struct Forbidden(pub String);
impl HttpApiError for Forbidden {
    fn code(&self) -> ErrorCode {
        ErrorCode::Forbidden
    }
}
impl_into_error_from_display!(Forbidden);

#[derive(Debug, thiserror::Error)]
#[error("Unknown database error")]
pub struct UnknownDbErr;
impl HttpApiError for UnknownDbErr {
    fn code(&self) -> ErrorCode {
        ErrorCode::DatabaseError
    }
}
impl_into_error_from_display!(UnknownDbErr);

#[derive(Debug, thiserror::Error)]
#[error("Workspace not initialized. Call `PUT {}` to initialize it.", uri!(crate::v1::init::init_workspace))]
pub struct WorkspaceNotInitialized;
impl HttpApiError for WorkspaceNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::WorkspaceNotInitialized
    }
}
impl_into_error_from_display!(WorkspaceNotInitialized);

#[derive(Debug, thiserror::Error)]
#[error("XMPP server not initialized. Call `PUT {}` to initialize it.", uri!(crate::v1::init::init_server_config))]
pub struct ServerConfigNotInitialized;
impl HttpApiError for ServerConfigNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::ServerConfigNotInitialized
    }
}
impl_into_error_from_display!(ServerConfigNotInitialized);

#[derive(Debug, thiserror::Error)]
#[error("Bad request: {reason}")]
pub struct BadRequest {
    pub reason: String,
}
impl HttpApiError for BadRequest {
    fn code(&self) -> ErrorCode {
        ErrorCode::BadRequest
    }
}
impl_into_error_from_display!(BadRequest);

#[derive(Debug, thiserror::Error)]
#[error("Not found: {reason}")]
pub struct NotFound {
    pub reason: String,
}
impl HttpApiError for NotFound {
    fn code(&self) -> ErrorCode {
        ErrorCode::NotFound
    }
}
impl_into_error_from_display!(NotFound);

/// HTTP status (used by the [default catcher](https://rocket.rs/guide/v0.5/requests/#default-catchers)
/// to change the output format).
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HTTPStatus(pub Status);
impl HttpApiError for HTTPStatus {
    fn code(&self) -> ErrorCode {
        ErrorCode::Unknown(self.0)
    }
}
impl_into_error_from_display!(HTTPStatus);

macro_rules! impl_into_error {
    ($t:ty) => {
        impl From<$t> for Error {
            fn from(error: $t) -> Self {
                Self::new(
                    error.code(),
                    format!("{} error: {error}", stringify!($t)),
                    error.headers(),
                )
            }
        }
    };
}

impl HttpApiError for sea_orm::DbErr {
    fn code(&self) -> ErrorCode {
        ErrorCode::DatabaseError
    }
}
impl_into_error!(sea_orm::DbErr);

impl HttpApiError for server_ctl::Error {
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}
impl_into_error!(server_ctl::Error);

impl HttpApiError for xmpp_service::Error {
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}
impl_into_error!(xmpp_service::Error);

impl HttpApiError for notifier::Error {
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}
impl_into_error!(notifier::Error);

impl HttpApiError for AuthBasicError {
    fn code(&self) -> ErrorCode {
        ErrorCode::Unauthorized
    }
    fn headers(&self) -> Vec<(String, String)> {
        vec![(
            "WWW-Authenticate".into(),
            r#"Basic realm="Admin only area", charset="UTF-8""#.into(),
        )]
    }
}
impl_into_error!(AuthBasicError);

impl HttpApiError for MutationError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EntityNotFound { .. } => ErrorCode::NotFound,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(MutationError);

impl HttpApiError for jwt_service::Error {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidJwtKey(_)
            | Self::CouldNotAddClaims(_)
            | Self::Encode(_)
            | Self::Other(_) => ErrorCode::InternalServerError,
            Self::Decode(_) | Self::InvalidClaim(_) => ErrorCode::Unauthorized,
        }
    }
}
impl_into_error!(jwt_service::Error);

impl HttpApiError for auth_service::Error {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidCredentials => ErrorCode::Unauthorized,
            _ => ErrorCode::InternalServerError,
        }
    }
}
impl_into_error!(auth_service::Error);

impl HttpApiError for UserCreateError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::InternalServerError,
        }
    }
}
impl_into_error!(UserCreateError);

impl HttpApiError for user_service::Error {
    fn code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateUser(err) => err.code(),
        }
    }
}
impl_into_error!(user_service::Error);

impl HttpApiError for server_manager::Error {
    fn code(&self) -> ErrorCode {
        match self {
            Self::ServerConfigAlreadyInitialized => ErrorCode::ServerConfigAlreadyInitialized,
            Self::ServerCtl(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(server_manager::Error);

impl HttpApiError for InitServerConfigError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::CouldNotInitServerConfig(err) => err.code(),
            Self::CouldNotCreateServiceAccount(err) => err.code(),
        }
    }
}
impl_into_error!(InitServerConfigError);

impl HttpApiError for CreateServiceAccountError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateXmppAccount(err) => err.code(),
            Self::CouldNotLogIn(_) | Self::InvalidJwt(_) | Self::MissingProsodyToken(_) => {
                ErrorCode::InternalServerError
            }
        }
    }
}
impl_into_error!(CreateServiceAccountError);

impl HttpApiError for InitWorkspaceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceAlreadyInitialized => ErrorCode::WorkspaceAlreadyInitialized,
            Self::XmppAccountNotInitialized => ErrorCode::ServerConfigNotInitialized,
            Self::CouldNotSetWorkspaceName(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InitWorkspaceError);

impl HttpApiError for InitFirstAccountError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::FirstAccountAlreadyCreated => ErrorCode::FirstAccountAlreadyCreated,
            Self::InvalidJid(_) => ErrorCode::BadRequest,
            Self::CouldNotCreateFirstAccount(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InitFirstAccountError);

impl HttpApiError for InviteMemberError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidJid(_) => ErrorCode::BadRequest,
            Self::CouldNotUpdateInvitationStatus { .. } => ErrorCode::InternalServerError,
            #[cfg(debug_assertions)]
            Self::CouldNotAutoAcceptInvitation(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InviteMemberError);

impl HttpApiError for invitation_service::InvitationAcceptError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::InternalServerError,
        }
    }
}
impl_into_error!(invitation_service::InvitationAcceptError);

impl HttpApiError for InvitationAcceptError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound => ErrorCode::Unauthorized,
            Self::ExpiredAcceptToken => ErrorCode::NotFound,
            Self::ServiceError(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationAcceptError);

impl HttpApiError for InvitationRejectError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound => ErrorCode::Unauthorized,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationRejectError);

impl HttpApiError for InvitationResendError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(_) => ErrorCode::NotFound,
            Self::CouldNotSendInvitation(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationResendError);

impl HttpApiError for InvitationCancelError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationCancelError);

impl HttpApiError for std::io::Error {
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}
impl_into_error!(std::io::Error);

impl HttpApiError for WorkspaceControllerError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotInitialized => ErrorCode::WorkspaceNotInitialized,
            Self::XmppServiceError(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(WorkspaceControllerError);

impl HttpApiError for WorkspaceControllerInitError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceXmppAccountNotInitialized => ErrorCode::ServerConfigNotInitialized,
        }
    }
}
impl_into_error!(WorkspaceControllerInitError);
