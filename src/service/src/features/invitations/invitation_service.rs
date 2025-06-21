// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use chrono::{DateTime, Utc};
use jid::DomainRef;
use sea_orm::{
    DatabaseConnection, DbConn, ItemsAndPagesNumber, ModelTrait as _, TransactionTrait as _,
};
use secrecy::{ExposeSecret as _, SecretString};
use tracing::{debug, error, warn};

use crate::{
    dependencies,
    invitations::{Invitation, InvitationRepository},
    members::{MemberRepository, MemberRole, UnauthenticatedMemberService, UserCreateError},
    notifications::{notifier::email::EmailNotification, NotificationService},
    onboarding,
    util::{bare_jid_from_username, either::Either3},
    workspace::WorkspaceService,
    xmpp::{BareJid, JidNode},
    AppConfig,
};

use super::{
    InvitationContact, InvitationCreateForm, InvitationStatus, InvitationToken,
    WorkspaceInvitationPayload,
};

#[derive(Debug, Clone)]
pub struct InvitationService {
    db: DatabaseConnection,
    uuid_gen: dependencies::Uuid,
    member_service: UnauthenticatedMemberService,
}

impl InvitationService {
    pub fn new(
        db: DatabaseConnection,
        uuid_gen: dependencies::Uuid,
        member_service: UnauthenticatedMemberService,
    ) -> Self {
        Self {
            db,
            uuid_gen,
            member_service,
        }
    }
}

impl InvitationService {
    pub async fn invite_member(
        &self,
        app_config: &AppConfig,
        server_domain: &DomainRef,
        notification_service: &NotificationService,
        workspace_service: &WorkspaceService,
        form: impl Into<InviteMemberForm>,
        #[cfg(debug_assertions)] auto_accept: bool,
    ) -> Result<Invitation, InviteMemberError> {
        let form = form.into();
        let jid = form.jid(server_domain);

        if (InvitationRepository::get_by_jid(&self.db, &jid).await)
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(InviteMemberError::InvitationConfict);
        }
        if (MemberRepository::get(&self.db, &jid).await)
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(InviteMemberError::UsernameConfict);
        }

        let invitation = InvitationRepository::create(
            &self.db,
            InvitationCreateForm {
                jid,
                pre_assigned_role: Some(form.pre_assigned_role.clone()),
                contact: form.contact.clone(),
                created_at: None,
            },
            &self.uuid_gen,
        )
        .await
        .context("Database error")?;

        (onboarding::at_least_one_invitation_sent::set(&self.db, true).await)
            .inspect_err(|err| warn!("Could not set `at_least_one_invitation_sent` to true: {err}"))
            .ok();

        let workspace_name = (workspace_service.get_workspace_name().await)
            .context("Could not get workspace details (to build the notification)")?;

        if let Err(err) = notification_service
            .send_workspace_invitation(
                form.contact,
                WorkspaceInvitationPayload {
                    accept_token: invitation.accept_token.into(),
                    reject_token: invitation.reject_token.into(),
                    workspace_name,
                    dashboard_url: app_config.pod.dashboard_url(),
                    api_app_name: app_config.branding.page_title.clone(),
                    organization_name: app_config.branding.company_name.clone(),
                },
                app_config,
            )
            .await
        {
            error!("Could not send workspace invitation: {err}");
            InvitationRepository::update_status(
                &self.db,
                invitation.clone(),
                InvitationStatus::SendFailed,
            )
            .await
            .map_or_else(
                |err| {
                    error!(
                        "Could not mark workspace invitation `{id}` as `{status}`: {err}",
                        id = invitation.id,
                        status = InvitationStatus::SendFailed,
                    )
                },
                |_| {
                    debug!(
                        "Marked invitation `{id}` as `{status}`",
                        id = invitation.id,
                        status = InvitationStatus::SendFailed,
                    )
                },
            );
        };

        InvitationRepository::update_status(&self.db, invitation.clone(), InvitationStatus::Sent)
            .await
            .context(format!(
                "Could not mark workspace invitation `{id}` as `{status}`",
                id = invitation.id,
                status = InvitationStatus::Sent,
            ))?;

        #[cfg(debug_assertions)]
        if auto_accept {
            warn!("As requested, the created invitation will be automatically accepted.");

            let password: SecretString = if app_config
                .debug_only
                .insecure_password_on_auto_accept_invitation
            {
                // Use JID as password to make password predictable
                invitation.jid.to_string().into()
            } else {
                crate::auth::util::strong_random_password(32)
            };
            self.accept_by_token(
                invitation.accept_token.into(),
                InvitationAcceptForm {
                    nickname: form.username.to_string(),
                    password,
                },
            )
            .await?;
        }

        Ok(invitation)
    }
}

impl NotificationService {
    async fn send_workspace_invitation(
        &self,
        contact: InvitationContact,
        payload: WorkspaceInvitationPayload,
        app_config: &AppConfig,
    ) -> Result<(), anyhow::Error> {
        match contact {
            InvitationContact::Email { email_address } => {
                let email =
                    EmailNotification::for_workspace_invitation(email_address, payload, app_config)
                        .context("Could not create email")?;
                self.send_email(email)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct InviteMemberForm {
    pub username: JidNode,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
}

impl InviteMemberForm {
    fn jid(&self, server_domain: &DomainRef) -> BareJid {
        bare_jid_from_username(&self.username, server_domain)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InviteMemberError {
    #[error("Invitation already exists (choose a different username).")]
    InvitationConfict,
    #[error("Username already taken.")]
    UsernameConfict,
    #[cfg(debug_assertions)]
    #[error("Could not auto-accept the invitation: {0}")]
    CouldNotAutoAcceptInvitation(#[from] CannotAcceptInvitation),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl InvitationService {
    pub async fn get(&self, id: &i32) -> Result<Option<Invitation>, anyhow::Error> {
        (InvitationRepository::get_by_id(&self.db, id).await).context("Database error")
    }
    pub async fn get_invitations(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), anyhow::Error> {
        (InvitationRepository::get_all(&self.db, page_number, page_size, until).await)
            .context("Database error")
    }
}

impl InvitationService {
    pub async fn accept(
        &self,
        db: &DbConn,
        invitation: Invitation,
        form: InvitationAcceptForm,
    ) -> Result<(), InvitationAcceptError> {
        let txn = db.begin().await.context("Database error")?;

        let email_address = match invitation.contact() {
            InvitationContact::Email { email_address } => Some(email_address),
        };

        // Create the user
        self.member_service
            .create_user(
                &txn,
                &invitation.jid,
                &form.password,
                &form.nickname,
                &Some(invitation.pre_assigned_role),
                email_address,
            )
            .await?;

        // Delete the invitation from database
        (InvitationRepository::accept(&txn, invitation).await)
            .context("Invitation repository could not accept the inviation")?;

        // Commit the transaction if everything went well
        txn.commit().await.context("Database error")?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationAcceptError {
    #[error("Could not create user: {0}")]
    CouldNotCreateUser(#[from] UserCreateError),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl InvitationService {
    pub async fn get_by_accept_token(
        &self,
        token: InvitationToken,
    ) -> Result<Invitation, Either3<NoInvitationForToken, InvitationExpired, anyhow::Error>> {
        match InvitationRepository::get_by_accept_token(&self.db, token).await? {
            Some(invitation) if invitation.is_expired() => Err(Either3::E2(InvitationExpired)),
            Some(invitation) => Ok(invitation),
            None => Err(Either3::E1(NoInvitationForToken)),
        }
    }
    pub async fn get_by_reject_token(
        &self,
        token: InvitationToken,
    ) -> Result<Invitation, Either3<NoInvitationForToken, InvitationExpired, anyhow::Error>> {
        match InvitationRepository::get_by_reject_token(&self.db, token).await? {
            Some(invitation) if invitation.is_expired() => Err(Either3::E2(InvitationExpired)),
            Some(invitation) => Ok(invitation),
            None => Err(Either3::E1(NoInvitationForToken)),
        }
    }
}

impl InvitationService {
    pub async fn accept_by_token(
        &self,
        token: InvitationToken,
        form: impl Into<InvitationAcceptForm>,
    ) -> Result<(), CannotAcceptInvitation> {
        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = self.get_by_accept_token(token.clone()).await?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.accept_token);

        if invitation.is_expired() {
            return Err(CannotAcceptInvitation::from(InvitationExpired));
        }

        // Check if JID is already taken (in which case the member cannot be created).
        // NOTE: There should not be any invitation for an already-taken username,
        //   but let's keep this as a safeguard.
        if (MemberRepository::get(&self.db, &invitation.jid).await)
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(CannotAcceptInvitation::MemberAlreadyExists);
        }

        (self.accept(&self.db, invitation, form.into()).await)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct InvitationAcceptForm {
    pub nickname: String,
    pub password: SecretString,
}

#[derive(Debug, thiserror::Error)]
#[error("Invitation expired.")]
pub struct InvitationExpired;

#[derive(Debug, thiserror::Error)]
#[error("No invitation found for provided token.")]
pub struct NoInvitationForToken;

#[derive(Debug, thiserror::Error)]
pub enum CannotAcceptInvitation {
    #[error("{0}")]
    InvitationNotFound(#[from] NoInvitationForToken),
    #[error("{0}")]
    InvitationExpired(#[from] InvitationExpired),
    #[error("Member already exists (JID already taken).")]
    MemberAlreadyExists,
    #[error("{0}")]
    AcceptError(#[from] InvitationAcceptError),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl InvitationService {
    pub async fn reject_by_token(
        &self,
        token: InvitationToken,
    ) -> Result<(), Either3<NoInvitationForToken, InvitationExpired, anyhow::Error>> {
        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = self.get_by_reject_token(token.clone()).await?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.reject_token);

        (invitation.delete(&self.db).await).context("Database error")?;

        Ok(())
    }
}

impl InvitationService {
    pub async fn resend(
        &self,
        app_config: &AppConfig,
        notification_service: &NotificationService,
        workspace_service: &WorkspaceService,
        invitation_id: i32,
    ) -> Result<(), InvitationResendError> {
        let invitation = (InvitationRepository::get_by_id(&self.db, &invitation_id).await)
            .context("Database error")?
            .ok_or(InvitationResendError::InvitationNotFound(invitation_id))?;

        let workspace = (workspace_service.get_workspace().await)
            .context("Could not get workspace details (to build the notification)")?;

        notification_service
            .send_workspace_invitation(
                invitation.contact(),
                WorkspaceInvitationPayload {
                    accept_token: invitation.accept_token.into(),
                    reject_token: invitation.reject_token.into(),
                    workspace_name: workspace.name.clone(),
                    dashboard_url: app_config.pod.dashboard_url().clone(),
                    api_app_name: app_config.branding.page_title.clone(),
                    organization_name: app_config.branding.company_name.clone(),
                },
                app_config,
            )
            .await
            .context("Could not send invitation")?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationResendError {
    #[error("Could not find the invitation with id '{0}'.")]
    InvitationNotFound(i32),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl InvitationService {
    pub async fn cancel(&self, invitation_id: i32) -> anyhow::Result<()> {
        (InvitationRepository::delete_by_id(&self.db, invitation_id).await)
            .context("Database error")?;

        Ok(())
    }
}

// MARK: - Boilerplate

impl<E1, E2, E3> From<Either3<E1, E2, E3>> for CannotAcceptInvitation
where
    CannotAcceptInvitation: From<E1> + From<E2> + From<E3>,
{
    fn from(value: Either3<E1, E2, E3>) -> Self {
        match value {
            Either3::E1(val) => Self::from(val),
            Either3::E2(val) => Self::from(val),
            Either3::E3(val) => Self::from(val),
        }
    }
}
