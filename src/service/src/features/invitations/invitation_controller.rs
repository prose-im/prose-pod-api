// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Context as _};
use jid::BareJid;
use serde::{Deserialize, Serialize};

use crate::{
    members::MemberRole,
    models::{Paginated, Pagination, PaginationForm},
    notifications::NotificationService,
    server_config::ServerConfig,
    util::{to_bare_jid, Either},
    workspace::WorkspaceService,
    AppConfig,
};

use super::{
    entities::workspace_invitation, CannotAcceptInvitation, Invitation, InvitationAcceptForm,
    InvitationId, InvitationRejectError, InvitationResendError, InvitationService, InvitationToken,
    InvitationTokenType, InviteMemberError, InviteMemberForm,
};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct InvitationNotFound(String);

// MARK: CREATE

#[cfg(not(debug_assertions))]
pub type InviteMemberResponse = Result<Invitation, InviteMemberError>;
#[cfg(not(debug_assertions))]
fn ok(invitation: Invitation) -> InviteMemberResponse {
    Ok(invitation)
}
#[cfg(debug_assertions)]
pub type InviteMemberResponse =
    Result<Either<Invitation, crate::members::Member>, Either<InviteMemberError, anyhow::Error>>;
#[cfg(debug_assertions)]
fn ok(invitation: Invitation) -> InviteMemberResponse {
    Ok(Either::Left(invitation))
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member(
    #[cfg(debug_assertions)] db: &sea_orm::DatabaseConnection,
    app_config: &AppConfig,
    server_config: &ServerConfig,
    notification_service: &NotificationService,
    invitation_service: &InvitationService,
    workspace_service: &WorkspaceService,
    #[cfg(debug_assertions)] auto_accept: bool,
    req: InviteMemberForm,
) -> InviteMemberResponse {
    let invitation = invitation_service
        .invite_member(
            app_config,
            server_config,
            notification_service,
            workspace_service,
            req,
            #[cfg(debug_assertions)]
            auto_accept,
        )
        .await
        .map_err(|err| {
            #[cfg(not(debug_assertions))]
            return err;
            #[cfg(debug_assertions)]
            return Either::Left(err);
        })?;

    #[cfg(debug_assertions)]
    {
        if auto_accept {
            let jid = invitation.jid;
            let member = (crate::members::MemberRepository::get(db, &jid).await)
                .context("Database error")
                .map_err(Either::Right)?
                .unwrap();
            return Ok(Either::Right(member.into()));
        }
    }

    ok(invitation)
}

// MARK: GET ONE

/// Get information about a workspace invitation.
pub async fn get_invitation(
    invitation_id: InvitationId,
    invitation_service: InvitationService,
) -> Result<Invitation, Either<InvitationNotFound, anyhow::Error>> {
    match invitation_service.get(&invitation_id).await {
        Ok(Some(invitation)) => Ok(invitation),
        Ok(None) => Err(Either::Left(InvitationNotFound(format!(
            "No invitation with id '{invitation_id}'.",
        )))),
        Err(err) => Err(Either::Right(anyhow!(err).context("Database error"))),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInvitationBasicDetails {
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
}

impl From<workspace_invitation::Model> for WorkspaceInvitationBasicDetails {
    fn from(value: workspace_invitation::Model) -> Self {
        Self {
            jid: to_bare_jid(&value.jid).unwrap(),
            pre_assigned_role: value.pre_assigned_role,
        }
    }
}

/// Get information about an invitation from an accept or reject token.
pub async fn get_invitation_by_token(
    token: InvitationToken,
    token_type: InvitationTokenType,
    invitation_service: &InvitationService,
) -> Result<WorkspaceInvitationBasicDetails, Either<InvitationNotFound, anyhow::Error>> {
    let res = match token_type {
        InvitationTokenType::Accept => invitation_service.get_by_accept_token(token).await,
        InvitationTokenType::Reject => invitation_service.get_by_reject_token(token).await,
    };

    match res {
        Ok(Some(invitation)) => Ok(invitation.into()),
        Ok(None) => Err(Either::Left(InvitationNotFound(
            "No invitation found for provided token.".to_string(),
        ))),
        Err(err) => Err(Either::Right(err)),
    }
}

// MARK: GET MANY

impl Pagination {
    fn invitations(
        PaginationForm {
            page_number,
            page_size,
            until,
        }: PaginationForm,
    ) -> Self {
        Self {
            page_number: page_number.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
            until,
        }
    }
}

/// Get workspace invitations.
pub async fn get_invitations(
    invitation_service: InvitationService,
    pagination: PaginationForm,
) -> anyhow::Result<Paginated<Invitation>> {
    let Pagination {
        page_number,
        page_size,
        until,
    } = Pagination::invitations(pagination);

    let (pages_metadata, invitations) = invitation_service
        .get_invitations(page_number, page_size, until)
        .await
        .context("Database error")?;

    Ok(Paginated::new(
        invitations,
        page_number,
        page_size,
        pages_metadata,
    ))
}

// MARK: ACTIONS

/// Accept a workspace invitation.
pub async fn invitation_accept(
    invitation_service: &InvitationService,
    token: InvitationToken,
    form: InvitationAcceptForm,
) -> Result<(), CannotAcceptInvitation> {
    invitation_service.accept_by_token(token, form).await
}

/// Reject a workspace invitation.
pub async fn invitation_reject(
    invitation_service: &InvitationService,
    token: InvitationToken,
) -> anyhow::Result<()> {
    match invitation_service.reject_by_token(token).await {
        Ok(()) => Ok(()),
        Err(InvitationRejectError::InvitationNotFound) => Ok(()),
        Err(InvitationRejectError::Internal(err)) => Err(err),
    }
}

/// Resend a workspace invitation.
pub async fn invitation_resend(
    invitation_service: &InvitationService,
    app_config: &AppConfig,
    notification_service: &NotificationService,
    workspace_service: &WorkspaceService,
    invitation_id: InvitationId,
) -> Result<(), InvitationResendError> {
    invitation_service
        .resend(
            app_config,
            notification_service,
            workspace_service,
            invitation_id,
        )
        .await
}

/// Cancel a workspace invitation.
pub async fn invitation_cancel(
    invitation_service: &InvitationService,
    invitation_id: InvitationId,
) -> anyhow::Result<()> {
    invitation_service.cancel(invitation_id).await
}
