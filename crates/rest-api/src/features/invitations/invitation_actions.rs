// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{response::status::NoContent, serde::json::Json, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    invitations::{invitation_service::*, InvitationAcceptError, InvitationToken},
    members::MemberRepository,
    notifications::Notifier,
    AppConfig,
};

use crate::{
    error::prelude::*,
    forms::Uuid,
    guards::{Db, LazyGuard},
    models::SerializableSecretString,
};

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: SerializableSecretString,
}

/// Accept a workspace invitation.
#[rocket::put(
    "/v1/invitation-tokens/<token>/accept",
    format = "json",
    data = "<req>"
)]
pub async fn invitation_accept_route<'r>(
    invitation_service: LazyGuard<InvitationService>,
    token: Uuid,
    req: Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    invitation_service
        .inner?
        .accept_by_token(InvitationToken::from(*token.deref()), req.into_inner())
        .await?;

    Ok(())
}

pub async fn invitation_accept_route_axum() {
    todo!()
}

/// Reject a workspace invitation.
#[rocket::put("/v1/invitation-tokens/<token>/reject")]
pub async fn invitation_reject_route<'r>(
    invitation_service: LazyGuard<InvitationService>,
    token: Uuid,
) -> Result<NoContent, Error> {
    invitation_service
        .inner?
        .reject_by_token(InvitationToken::from(*token.deref()))
        .await?;

    Ok(NoContent)
}

pub async fn invitation_reject_route_axum() {
    todo!()
}

/// Resend a workspace invitation.
#[rocket::post("/v1/invitations/<invitation_id>/resend")]
pub async fn invitation_resend_route<'r>(
    conn: Connection<'r, Db>,
    invitation_service: LazyGuard<InvitationService>,
    app_config: &State<AppConfig>,
    jid: LazyGuard<UserInfo>,
    notifier: LazyGuard<Notifier>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_service = invitation_service.inner?;
    let notifier = notifier.inner?;

    let jid = jid.inner?.jid;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_service
        .resend(&app_config, &notifier, invitation_id)
        .await?;

    Ok(NoContent)
}

pub async fn invitation_resend_route_axum() {
    todo!()
}

/// Cancel a workspace invitation.
#[rocket::delete("/v1/invitations/<invitation_id>")]
pub async fn invitation_cancel_route<'r>(
    conn: Connection<'r, Db>,
    invitation_service: LazyGuard<InvitationService>,
    user_info: LazyGuard<UserInfo>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_service = invitation_service.inner?;

    let jid = user_info.inner?.jid;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_service.cancel(invitation_id).await?;

    Ok(NoContent)
}

pub async fn invitation_cancel_route_axum() {
    todo!()
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

impl CustomErrorCode for InvitationResendError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound(_) => ErrorCode::NOT_FOUND,
            Self::CouldNotSendInvitation(err) => err.code(),
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
