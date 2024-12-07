// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::{DateTime, Utc};
#[cfg(debug_assertions)]
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{DatabaseConnection, DbErr, ItemsAndPagesNumber, ModelTrait as _};
use secrecy::{ExposeSecret as _, SecretString};
use tracing::{debug, error, warn};

use crate::{
    dependencies,
    invitations::Invitation,
    members::{MemberRepository, MemberRole},
    models::{BareJid, JidNode},
    notifications::{notifier, Notifier},
    server_config::ServerConfig,
    util::bare_jid_from_username,
    AppConfig, MutationError,
};

use super::{
    invitation_service, InvitationContact, InvitationCreateForm, InvitationRepository,
    InvitationService, InvitationStatus, InvitationToken,
};

#[derive(Clone)]
pub struct InvitationController {
    pub db: Arc<DatabaseConnection>,
    pub uuid_gen: Arc<dependencies::Uuid>,
}

impl InvitationController {
    pub async fn invite_member(
        &self,
        app_config: &AppConfig,
        server_config: &ServerConfig,
        notifier: &Notifier,
        form: impl Into<InviteMemberForm>,
        #[cfg(debug_assertions)] invitation_service: &InvitationService,
    ) -> Result<Invitation, InviteMemberError> {
        let form = form.into();
        let jid = form.jid(&server_config)?;

        if InvitationRepository::get_by_jid(self.db.as_ref(), &jid)
            .await
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(InviteMemberError::InvitationConfict);
        }

        let invitation = InvitationRepository::create(
            self.db.as_ref(),
            InvitationCreateForm {
                jid,
                pre_assigned_role: Some(form.pre_assigned_role.clone()),
                contact: form.contact.clone(),
                created_at: None,
            },
            &self.uuid_gen,
        )
        .await?;

        if let Err(err) = notifier
            .send_workspace_invitation(
                &app_config.branding,
                &invitation.accept_token.into(),
                &invitation.reject_token.into(),
            )
            .await
        {
            error!("Could not send workspace invitation: {err}");
            InvitationRepository::update_status(
                self.db.as_ref(),
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

        InvitationRepository::update_status(
            self.db.as_ref(),
            invitation.clone(),
            InvitationStatus::Sent,
        )
        .await
        .map_err(|err| InviteMemberError::CouldNotUpdateInvitationStatus {
            id: invitation.id,
            status: InvitationStatus::Sent,
            err,
        })?;

        #[cfg(debug_assertions)]
        if app_config.debug_only.automatically_accept_invitations {
            warn!(
                "Config `{}` is turned on. The created invitation will be automatically accepted.",
                stringify!(debug_only.automatically_accept_invitations),
            );

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
            self.accept(
                invitation.accept_token.into(),
                invitation_service,
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
    #[error("Could not mark workspace invitation `{id}` as `{status}`: {err}")]
    CouldNotUpdateInvitationStatus {
        id: i32,
        status: InvitationStatus,
        err: MutationError,
    },
    #[cfg(debug_assertions)]
    #[error("Could not auto-accept the invitation: {0}")]
    CouldNotAutoAcceptInvitation(#[from] CannotAcceptInvitation),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn get_by_accept_token(
        &self,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_accept_token(self.db.as_ref(), token).await
    }
    pub async fn get_by_reject_token(
        &self,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_reject_token(self.db.as_ref(), token).await
    }
}

impl InvitationController {
    pub async fn accept(
        &self,
        token: InvitationToken,
        invitation_service: &InvitationService,
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
        if MemberRepository::get(self.db.as_ref(), &invitation.jid)
            .await
            .as_ref()
            .is_ok_and(Option::is_some)
        {
            return Err(CannotAcceptInvitation::MemberAlreadyExists);
        }

        invitation_service
            .accept(self.db.as_ref(), invitation, &form.password, &form.nickname)
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
    ServiceError(#[from] invitation_service::InvitationAcceptError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn reject(&self, token: InvitationToken) -> Result<(), InvitationRejectError> {
        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = self
            .get_by_reject_token(token.clone())
            .await?
            .ok_or(InvitationRejectError::InvitationNotFound)?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.reject_token);

        invitation.delete(self.db.as_ref()).await?;

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

impl InvitationController {
    pub async fn resend(
        &self,
        config: &AppConfig,
        notifier: &Notifier,
        invitation_id: i32,
    ) -> Result<(), InvitationResendError> {
        let invitation = InvitationRepository::get_by_id(self.db.as_ref(), &invitation_id)
            .await?
            .ok_or(InvitationResendError::InvitationNotFound(invitation_id))?;

        notifier
            .send_workspace_invitation(
                &config.branding,
                &invitation.accept_token.into(),
                &invitation.reject_token.into(),
            )
            .await
            .map_err(InvitationResendError::CouldNotSendInvitation)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationResendError {
    #[error("Could not find the invitation with id '{0}'.")]
    InvitationNotFound(i32),
    #[error("Could not send invitation: {0}")]
    CouldNotSendInvitation(notifier::Error),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn cancel(&self, invitation_id: i32) -> Result<(), InvitationCancelError> {
        InvitationRepository::delete_by_id(self.db.as_ref(), invitation_id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationCancelError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn get_invitations(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), DbErr> {
        InvitationRepository::get_all(self.db.as_ref(), page_number, page_size, until).await
    }
}
