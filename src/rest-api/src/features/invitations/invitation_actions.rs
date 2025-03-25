// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use service::{
    invitations::{invitation_service::*, InvitationAcceptError, InvitationToken},
    models::SerializableSecretString,
    notifications::NotificationService,
    workspace::WorkspaceService,
    AppConfig,
};

use crate::error::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: SerializableSecretString,
}

/// Accept a workspace invitation.
pub async fn invitation_accept_route(
    invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
    Json(req): Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    invitation_service.accept_by_token(token, req).await?;
    Ok(())
}

/// Reject a workspace invitation.
pub async fn invitation_reject_route(
    invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
) -> Result<StatusCode, Error> {
    invitation_service.reject_by_token(token).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Resend a workspace invitation.
pub async fn invitation_resend_route(
    invitation_service: InvitationService,
    app_config: AppConfig,
    notification_service: NotificationService,
    workspace_service: WorkspaceService,
    Path(invitation_id): Path<i32>,
) -> Result<StatusCode, Error> {
    invitation_service
        .resend(
            &app_config,
            &notification_service,
            &workspace_service,
            invitation_id,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Cancel a workspace invitation.
pub async fn invitation_cancel_route(
    invitation_service: InvitationService,
    Path(invitation_id): Path<i32>,
) -> Result<StatusCode, Error> {
    invitation_service.cancel(invitation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ERRORS

impl CustomErrorCode for InvitationAcceptError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(InvitationAcceptError);

impl CustomErrorCode for CannotAcceptInvitation {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound => ErrorCode::UNAUTHORIZED,
            Self::ExpiredAcceptToken => ErrorCode::NOT_FOUND,
            Self::MemberAlreadyExists => ErrorCode::MEMBER_ALREADY_EXISTS,
            Self::ServiceError(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(CannotAcceptInvitation);

impl CustomErrorCode for InvitationRejectError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound => ErrorCode::UNAUTHORIZED,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationRejectError);

impl CustomErrorCode for SendWorkspaceInvitationError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CouldNotCreateEmailNotification(_) => ErrorCode::INTERNAL_SERVER_ERROR,
            Self::NotificationService(err) => err.code(),
        }
    }
}
impl_into_error!(SendWorkspaceInvitationError);

impl CustomErrorCode for InvitationResendError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(_) => ErrorCode::NOT_FOUND,
            Self::CouldNotSendInvitation(err) => err.code(),
            Self::CouldNotGetWorkspaceDetails(_) => ErrorCode::INTERNAL_SERVER_ERROR,
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationResendError);

impl CustomErrorCode for InvitationCancelError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InvitationCancelError);

// BOILERPLATE

impl Into<InvitationAcceptForm> for AcceptWorkspaceInvitationRequest {
    fn into(self) -> InvitationAcceptForm {
        InvitationAcceptForm {
            nickname: self.nickname,
            password: self.password.into(),
        }
    }
}
