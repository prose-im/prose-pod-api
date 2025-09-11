// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Context as _};
use jid::{BareJid, DomainRef};
use serdev::Serialize;

use crate::{
    members::MemberRole,
    models::{Paginated, Pagination, PaginationForm},
    notifications::NotificationService,
    util::either::{Either, Either3},
    workspace::WorkspaceService,
    AppConfig,
};

use super::{
    entities::workspace_invitation, CannotAcceptInvitation, Invitation, InvitationAcceptForm,
    InvitationExpired, InvitationId, InvitationResendError, InvitationService, InvitationToken,
    InvitationTokenType, InviteMemberError, InviteMemberForm, NoInvitationForToken,
};

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("No invitation with id '{0}'.")]
pub struct InvitationNotFound(InvitationId);

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
    Ok(Either::E1(invitation))
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member(
    #[cfg(debug_assertions)] db: &sea_orm::DatabaseConnection,
    app_config: &AppConfig,
    server_domain: &DomainRef,
    notification_service: &NotificationService,
    invitation_service: &InvitationService,
    workspace_service: &WorkspaceService,
    #[cfg(debug_assertions)] auto_accept: bool,
    req: InviteMemberForm,
) -> InviteMemberResponse {
    let invitation = invitation_service
        .invite_member(
            app_config,
            server_domain,
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
            return Either::E1(err);
        })?;

    #[cfg(debug_assertions)]
    {
        if auto_accept {
            let jid = invitation.jid;
            let member = (crate::members::MemberRepository::get(db, &jid).await)
                .context("Database error")
                .map_err(Either::E2)?
                .unwrap();
            return Ok(Either::E2(member.into()));
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
        Ok(None) => Err(Either::E1(InvitationNotFound(invitation_id))),
        Err(err) => Err(Either::E2(anyhow!(err).context("Database error"))),
    }
}

#[derive(Clone, Debug)]
#[derive(Serialize)]
pub struct WorkspaceInvitationBasicDetails {
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub is_expired: bool,
}

impl From<workspace_invitation::Model> for WorkspaceInvitationBasicDetails {
    fn from(value: workspace_invitation::Model) -> Self {
        Self {
            is_expired: value.is_expired(),
            jid: value.jid.into(),
            pre_assigned_role: value.pre_assigned_role,
        }
    }
}

/// Get information about an invitation from an accept or reject token.
pub async fn get_invitation_by_token(
    token: InvitationToken,
    token_type: InvitationTokenType,
    invitation_service: &InvitationService,
) -> Result<
    WorkspaceInvitationBasicDetails,
    Either3<NoInvitationForToken, InvitationExpired, anyhow::Error>,
> {
    let res = match token_type {
        InvitationTokenType::Accept => invitation_service.get_by_accept_token(token).await,
        InvitationTokenType::Reject => invitation_service.get_by_reject_token(token).await,
    };

    match res {
        Ok(invitation) => Ok(invitation.into()),
        Err(err) => Err(err),
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
        Err(Either3::E1(NoInvitationForToken)) => Ok(()),
        Err(Either3::E2(InvitationExpired)) => Ok(()),
        Err(Either3::E3(err)) => Err(err),
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
