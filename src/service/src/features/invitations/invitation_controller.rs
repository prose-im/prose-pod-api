// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{
    auth::AuthToken,
    errors::Forbidden,
    invitations::{
        invitation_service::{AcceptAccountInvitationCommand, InviteUserError},
        InvitationRepository,
    },
    licensing::errors::UserLimitReached,
    members::Member,
    models::{Paginated, Pagination, PaginationForm},
    util::either::{Either, Either3, Either4},
};

use super::{errors::*, models::*, InvitationService};

// MARK: Create

#[cfg(not(debug_assertions))]
pub type InviteMemberResponse = Invitation;
#[cfg(not(debug_assertions))]
fn ok<E>(invitation: Invitation) -> Result<InviteMemberResponse, E> {
    Ok(invitation)
}
#[cfg(debug_assertions)]
pub type InviteMemberResponse = Either<Invitation, crate::members::Member>;
#[cfg(debug_assertions)]
fn ok<E>(invitation: Invitation) -> Result<InviteMemberResponse, E> {
    Ok(Either::E1(invitation))
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member(
    invitation_service: &InvitationService,
    #[cfg(debug_assertions)] app_config: &crate::AppConfig,
    auth: &AuthToken,
    #[cfg(debug_assertions)] auto_accept: bool,
    req: InviteMemberForm,
) -> Result<InviteMemberResponse, InviteUserError> {
    #[cfg(debug_assertions)]
    let username = req.username.clone();

    let email_address = match req.contact {
        InvitationContact::Email { email_address } => email_address,
    };

    let invitation = invitation_service
        .invite_user(
            super::invitation_service::InviteUserCommand {
                username: req.username,
                role: req.pre_assigned_role,
                email_address,
                ttl: None,
            },
            auth,
        )
        .await?;

    #[cfg(debug_assertions)]
    if auto_accept {
        use crate::{
            invitations::invitation_service::AcceptAccountInvitationCommand, models::EmailAddress,
        };

        tracing::warn!("As requested, the created invitation will be automatically accepted.");

        let password: secrecy::SecretString = if app_config
            .debug_only
            .insecure_password_on_auto_accept_invitation
        {
            // Use JID as password to make password predictable
            invitation.jid.to_string().into()
        } else {
            crate::auth::util::random_secret(32)
        };

        let member = invitation_service
            .accept_account_invitation(
                invitation.accept_token,
                AcceptAccountInvitationCommand {
                    nickname: username.into(),
                    password,
                    email: Some(EmailAddress::from(&invitation.jid)),
                },
            )
            .await
            .map_err(|err| InviteUserError::Internal(anyhow::Error::new(err)))?;

        return Ok(Either::E2(member));
    }

    ok(invitation)
}

// MARK: Get one

/// Get information about a workspace invitation.
pub async fn get_invitation(
    invitation_repository: &InvitationRepository,
    invitation_id: &InvitationId,
    auth: &AuthToken,
) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>> {
    invitation_repository
        .get_account_invitation_by_id(invitation_id, auth)
        .await
}

/// Get information about an invitation from an accept or reject token.
pub async fn get_invitation_by_token(
    invitation_repository: &InvitationRepository,
    token: &InvitationToken,
) -> Result<Option<WorkspaceInvitationBasicDetails>, anyhow::Error> {
    match invitation_repository
        .get_account_invitation_by_token(token)
        .await
    {
        Ok(Some(invitation)) => Ok(Some(WorkspaceInvitationBasicDetails::from(invitation))),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

// MARK: Get many

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

pub async fn get_invitations(
    invitation_repository: &InvitationRepository,
    pagination: PaginationForm,
    auth: &AuthToken,
) -> Result<Paginated<Invitation>, Either<Forbidden, anyhow::Error>> {
    let Pagination {
        page_number,
        page_size,
        until,
    } = Pagination::invitations(pagination);

    let (pages_metadata, invitations) = invitation_repository
        .list_account_invitations_paged(page_number, page_size, until, auth)
        .await?;

    Ok(Paginated::new(
        invitations,
        page_number,
        page_size,
        pages_metadata,
    ))
}

// MARK: Actions

/// Accept a workspace invitation.
pub async fn invitation_accept(
    invitation_service: &InvitationService,
    token: InvitationToken,
    command: AcceptAccountInvitationCommand,
) -> Result<
    Member,
    Either4<UserLimitReached, InvitationNotFoundForToken, MemberAlreadyExists, anyhow::Error>,
> {
    invitation_service
        .accept_account_invitation(token, command)
        .await
}

/// Reject a workspace invitation.
pub async fn invitation_reject(
    invitation_service: &InvitationService,
    token: InvitationToken,
) -> anyhow::Result<()> {
    invitation_service.reject_account_invitation(token).await
}

/// Resend a workspace invitation.
pub async fn invitation_resend(
    invitation_service: &InvitationService,
    invitation_id: &InvitationId,
    auth: &AuthToken,
) -> Result<(), Either3<Forbidden, InvitationNotFound, anyhow::Error>> {
    invitation_service
        .resend_account_invitation(invitation_id, auth)
        .await
}

/// Cancel a workspace invitation.
pub async fn invitation_cancel(
    invitation_service: &InvitationService,
    invitation_id: InvitationId,
    auth: &AuthToken,
) -> Result<(), Either<Forbidden, anyhow::Error>> {
    invitation_service
        .cancel_invitation(invitation_id, auth)
        .await
}
