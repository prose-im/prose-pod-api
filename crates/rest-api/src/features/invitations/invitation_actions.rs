// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{delete, post, response::status::NoContent, serde::json::Json, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    features::{
        invitations::{invitation_controller::*, invitation_service, InvitationToken},
        members::MemberRepository,
        notifications::Notifier,
    },
    models::BareJid,
    AppConfig,
};

use crate::{
    error::prelude::*,
    forms::Uuid,
    guards::{Db, LazyGuard},
    models::SerializableSecretString,
};

use super::guards::*;

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: SerializableSecretString,
}

/// Accept a workspace invitation.
#[put("/v1/invitations/<token>/accept", format = "json", data = "<req>")]
pub async fn invitation_accept_route<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    invitation_service: LazyGuard<UnauthenticatedInvitationService<'r>>,
    token: Uuid,
    req: Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    invitation_controller
        .inner?
        .accept(
            InvitationToken::from(*token.deref()),
            invitation_service.inner?.deref(),
            req.into_inner(),
        )
        .await?;

    Ok(())
}

/// Reject a workspace invitation.
#[put("/v1/invitations/<token>/reject")]
pub async fn invitation_reject_route<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    token: Uuid,
) -> Result<NoContent, Error> {
    invitation_controller
        .inner?
        .reject(InvitationToken::from(*token.deref()))
        .await?;

    Ok(NoContent)
}

/// Resend a workspace invitation.
#[post("/v1/invitations/<invitation_id>/resend")]
pub async fn invitation_resend_route<'r>(
    conn: Connection<'r, Db>,
    invitation_controller: LazyGuard<InvitationController<'r>>,
    app_config: &State<AppConfig>,
    jid: LazyGuard<BareJid>,
    notifier: LazyGuard<Notifier<'r>>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_controller = invitation_controller.inner?;
    let notifier = notifier.inner?;

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_controller
        .resend(&app_config, &notifier, invitation_id)
        .await?;

    Ok(NoContent)
}

/// Cancel a workspace invitation.
#[delete("/v1/invitations/<invitation_id>")]
pub async fn invitation_cancel_route<'r>(
    conn: Connection<'r, Db>,
    invitation_controller: LazyGuard<InvitationController<'r>>,
    jid: LazyGuard<BareJid>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_controller = invitation_controller.inner?;

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_controller.cancel(invitation_id).await?;

    Ok(NoContent)
}

// ERRORS

impl CustomErrorCode for invitation_service::InvitationAcceptError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            _ => ErrorCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl_into_error!(invitation_service::InvitationAcceptError);

impl CustomErrorCode for CannotAcceptInvitation {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvitationNotFound => ErrorCode::UNAUTHORIZED,
            Self::ExpiredAcceptToken => ErrorCode::NOT_FOUND,
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
