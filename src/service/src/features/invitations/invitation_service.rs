// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
#[cfg(debug_assertions)]
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{
    DatabaseConnection, DbConn, DbErr, ItemsAndPagesNumber, ModelTrait as _, TransactionTrait as _,
};
use secrecy::{ExposeSecret as _, SecretString};
use tracing::{debug, error, warn};

use crate::{
    dependencies,
    invitations::{Invitation, InvitationRepository},
    members::{MemberRepository, MemberRole, UnauthenticatedMemberService, UserCreateError},
    notifications::{
        notification_service,
        notifier::email::{EmailNotification, EmailNotificationCreateError},
        NotificationService,
    },
    onboarding,
    pod_config::{PodConfigField, PodConfigRepository},
    server_config::ServerConfig,
    util::bare_jid_from_username,
    workspace::{WorkspaceService, WorkspaceServiceError},
    xmpp::{BareJid, JidNode},
    AppConfig, MutationError,
};

use super::{
    InvitationContact, InvitationCreateForm, InvitationId, InvitationStatus, InvitationToken,
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
        server_config: &ServerConfig,
        notification_service: &NotificationService,
        workspace_service: &WorkspaceService,
        form: impl Into<InviteMemberForm>,
        #[cfg(debug_assertions)] auto_accept: bool,
    ) -> Result<Invitation, InviteMemberError> {
        let form = form.into();
        let jid = form.jid(&server_config)?;

        if InvitationRepository::get_by_jid(&self.db, &jid)
            .await
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(InviteMemberError::InvitationConfict);
        }
        if MemberRepository::get(&self.db, &jid)
            .await
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
        .await?;

        (onboarding::at_least_one_invitation_sent::set(&self.db, true).await)
            .inspect_err(|err| warn!("Could not set `at_least_one_invitation_sent` to true: {err}"))
            .ok();

        let workspace_name = workspace_service
            .get_workspace_name()
            .await
            .map_err(InviteMemberError::CouldNotGetWorkspaceDetails)?;

        let dashboard_url = PodConfigRepository::get_dashboard_url(&self.db)
            .await
            .map_err(InviteMemberError::DbErr)?
            .ok_or(InviteMemberError::PodConfigMissing(
                PodConfigField::DashboardUrl,
            ))?;

        if let Err(err) = notification_service
            .send_workspace_invitation(
                form.contact,
                WorkspaceInvitationPayload {
                    accept_token: invitation.accept_token.into(),
                    reject_token: invitation.reject_token.into(),
                    workspace_name,
                    dashboard_url,
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
                        "Could not mark workspace invitation `{}` as `{}`: {err}",
                        invitation.id,
                        InvitationStatus::SendFailed
                    )
                },
                |_| {
                    debug!(
                        "Marked invitation `{}` as `{}`",
                        invitation.id,
                        InvitationStatus::SendFailed
                    )
                },
            );
        };

        InvitationRepository::update_status(&self.db, invitation.clone(), InvitationStatus::Sent)
            .await
            .map_err(|err| InviteMemberError::CouldNotUpdateInvitationStatus {
                id: invitation.id,
                status: InvitationStatus::Sent,
                err,
            })?;

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
                // NOTE: Code taken from <https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters>.
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect::<String>()
                    .into()
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
    ) -> Result<(), SendWorkspaceInvitationError> {
        match contact {
            InvitationContact::Email { email_address } => {
                self.send_email(EmailNotification::from(
                    email_address.into(),
                    payload,
                    app_config,
                )?)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendWorkspaceInvitationError {
    #[error("Could not create e-mail: {0}")]
    CouldNotCreateEmailNotification(#[from] EmailNotificationCreateError),
    #[error("`NotificationService` error: {0}")]
    NotificationService(#[from] notification_service::Error),
}

#[derive(Debug)]
pub struct InviteMemberForm {
    pub username: JidNode,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
}

impl InviteMemberForm {
    fn jid(&self, server_config: &ServerConfig) -> Result<BareJid, InviteMemberError> {
        bare_jid_from_username(&self.username, server_config).map_err(InviteMemberError::InvalidJid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InviteMemberError {
    #[error("Invalid JID: {0}")]
    InvalidJid(String),
    #[error("Invitation already exists (choose a different username).")]
    InvitationConfict,
    #[error("Username already taken.")]
    UsernameConfict,
    #[error("Could not send notification: {0}")]
    CouldNotSendNotification(#[from] SendWorkspaceInvitationError),
    #[error("Could not mark workspace invitation `{id}` as `{status}`: {err}")]
    CouldNotUpdateInvitationStatus {
        id: InvitationId,
        status: InvitationStatus,
        err: MutationError,
    },
    #[cfg(debug_assertions)]
    #[error("Could not auto-accept the invitation: {0}")]
    CouldNotAutoAcceptInvitation(#[from] CannotAcceptInvitation),
    #[error("Could not get workspace details (to build the notification): {0}")]
    CouldNotGetWorkspaceDetails(WorkspaceServiceError),
    #[error("Pod configuration missing: {0}")]
    PodConfigMissing(PodConfigField),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationService {
    pub async fn get(&self, id: &i32) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_id(&self.db, id).await
    }
    pub async fn get_invitations(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), DbErr> {
        InvitationRepository::get_all(&self.db, page_number, page_size, until).await
    }
}

impl InvitationService {
    pub async fn accept(
        &self,
        db: &DbConn,
        invitation: Invitation,
        password: &SecretString,
        nickname: &str,
    ) -> Result<(), InvitationAcceptError> {
        let txn = db.begin().await?;

        // Create the user
        self.member_service
            .create_user(
                &txn,
                &invitation.jid,
                &password,
                nickname,
                &Some(invitation.pre_assigned_role),
            )
            .await?;

        // Delete the invitation from database
        InvitationRepository::accept(&txn, invitation).await?;

        // Commit the transaction if everything went well
        txn.commit().await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationAcceptError {
    #[error("Could not create user: {0}")]
    CouldNotCreateUser(#[from] UserCreateError),
    #[error("Invitation repository could not accept the inviation: {0}")]
    CouldNotAcceptInvitation(#[from] MutationError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationService {
    pub async fn get_by_accept_token(
        &self,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_accept_token(&self.db, token).await
    }
    pub async fn get_by_reject_token(
        &self,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_reject_token(&self.db, token).await
    }
}

impl InvitationService {
    pub async fn accept_by_token(
        &self,
        token: InvitationToken,
        form: impl Into<InvitationAcceptForm>,
    ) -> Result<(), CannotAcceptInvitation> {
        let form = form.into();

        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = self
            .get_by_accept_token(token.clone())
            .await?
            .ok_or(CannotAcceptInvitation::InvitationNotFound)?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.accept_token);

        if invitation.accept_token_expires_at < Utc::now() {
            return Err(CannotAcceptInvitation::ExpiredAcceptToken);
        }

        // Check if JID is already taken (in which case the member cannot be created).
        // NOTE: There should not be any invitation for an already-taken username,
        //   but let's keep this as a safeguard.
        if MemberRepository::get(&self.db, &invitation.jid)
            .await
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(CannotAcceptInvitation::MemberAlreadyExists);
        }

        self.accept(&self.db, invitation, &form.password, &form.nickname)
            .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct InvitationAcceptForm {
    pub nickname: String,
    pub password: SecretString,
}

#[derive(Debug, thiserror::Error)]
pub enum CannotAcceptInvitation {
    #[error("No invitation found for provided token.")]
    InvitationNotFound,
    #[error("Invitation accept token has expired.")]
    ExpiredAcceptToken,
    #[error("Member already exists (JID already taken).")]
    MemberAlreadyExists,
    #[error("{0}")]
    ServiceError(#[from] InvitationAcceptError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationService {
    pub async fn reject_by_token(
        &self,
        token: InvitationToken,
    ) -> Result<(), InvitationRejectError> {
        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = self
            .get_by_reject_token(token.clone())
            .await?
            .ok_or(InvitationRejectError::InvitationNotFound)?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.reject_token);

        invitation.delete(&self.db).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationRejectError {
    #[error("No invitation found for provided token.")]
    InvitationNotFound,
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationService {
    pub async fn resend(
        &self,
        app_config: &AppConfig,
        notification_service: &NotificationService,
        workspace_service: &WorkspaceService,
        invitation_id: i32,
    ) -> Result<(), InvitationResendError> {
        let invitation = InvitationRepository::get_by_id(&self.db, &invitation_id)
            .await?
            .ok_or(InvitationResendError::InvitationNotFound(invitation_id))?;

        let workspace = workspace_service
            .get_workspace()
            .await
            .map_err(InvitationResendError::CouldNotGetWorkspaceDetails)?;

        let dashboard_url = PodConfigRepository::get_dashboard_url(&self.db)
            .await
            .map_err(InvitationResendError::DbErr)?
            .ok_or(InvitationResendError::PodConfigMissing(
                PodConfigField::DashboardUrl,
            ))?;

        notification_service
            .send_workspace_invitation(
                invitation.contact(),
                WorkspaceInvitationPayload {
                    accept_token: invitation.accept_token.into(),
                    reject_token: invitation.reject_token.into(),
                    workspace_name: workspace.name.clone(),
                    dashboard_url,
                    api_app_name: app_config.branding.page_title.clone(),
                    organization_name: app_config.branding.company_name.clone(),
                },
                app_config,
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationResendError {
    #[error("Could not find the invitation with id '{0}'.")]
    InvitationNotFound(i32),
    #[error("Could not send invitation: {0}")]
    CouldNotSendInvitation(#[from] SendWorkspaceInvitationError),
    #[error("Could not get workspace details (to build the notification): {0}")]
    CouldNotGetWorkspaceDetails(WorkspaceServiceError),
    #[error("Pod configuration missing: {0}")]
    PodConfigMissing(PodConfigField),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationService {
    pub async fn cancel(&self, invitation_id: i32) -> Result<(), InvitationCancelError> {
        InvitationRepository::delete_by_id(&self.db, invitation_id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationCancelError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
