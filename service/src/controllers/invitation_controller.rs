use chrono::{DateTime, Utc};
use entity::model::{InvitationContact, InvitationStatus, JIDNode, MemberRole};
use log::{debug, error, warn};
use prose_xmpp::BareJid;
#[cfg(debug_assertions)]
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{DatabaseConnection, DbErr, ItemsAndPagesNumber, ModelTrait as _};
use secrecy::{ExposeSecret as _, SecretString};

use crate::{
    config::Config as AppConfig,
    dependencies,
    model::{Invitation, ServerConfig},
    repositories::{InvitationCreateForm, InvitationRepository, InvitationToken},
    services::{
        invitation_service::{self, InvitationService},
        notifier::{self, Notifier},
    },
    util::bare_jid_from_username,
    MutationError,
};

pub enum InvitationController {}

impl InvitationController {
    pub async fn invite_member<'r>(
        db: &DatabaseConnection,
        uuid_gen: &dependencies::Uuid,
        app_config: &AppConfig,
        server_config: &ServerConfig,
        notifier: &Notifier<'r>,
        form: impl Into<InviteMemberForm>,
        #[cfg(debug_assertions)] invitation_service: &InvitationService<'r>,
    ) -> Result<Invitation, InviteMemberError> {
        let form = form.into();

        let invitation = InvitationRepository::create(
            db,
            InvitationCreateForm {
                jid: form.jid(&server_config)?,
                pre_assigned_role: Some(form.pre_assigned_role.clone()),
                contact: form.contact.clone(),
                created_at: None,
            },
            &uuid_gen,
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
                db,
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

        InvitationRepository::update_status(db, invitation.clone(), InvitationStatus::Sent)
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
            Self::accept(
                db,
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
    pub username: JIDNode,
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
    #[error("Could not mark workspace invitation `{id}` as `{status}`: {err}")]
    CouldNotUpdateInvitationStatus {
        id: i32,
        status: InvitationStatus,
        err: MutationError,
    },
    #[cfg(debug_assertions)]
    #[error("Could not auto-accept the invitation: {0}")]
    CouldNotAutoAcceptInvitation(#[from] InvitationAcceptError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn get_by_accept_token<'r>(
        db: &DatabaseConnection,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_accept_token(db, token).await
    }
    pub async fn get_by_reject_token<'r>(
        db: &DatabaseConnection,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        InvitationRepository::get_by_reject_token(db, token).await
    }
}

impl InvitationController {
    pub async fn accept<'r>(
        db: &DatabaseConnection,
        token: InvitationToken,
        invitation_service: &InvitationService<'r>,
        form: impl Into<InvitationAcceptForm>,
    ) -> Result<(), InvitationAcceptError> {
        let form = form.into();

        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = Self::get_by_accept_token(db, token.clone())
            .await?
            .ok_or(InvitationAcceptError::InvitationNotFound)?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.accept_token);

        if invitation.accept_token_expires_at < Utc::now() {
            return Err(InvitationAcceptError::ExpiredAcceptToken);
        }

        invitation_service
            .accept(db, invitation, &form.password, &form.nickname)
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
pub enum InvitationAcceptError {
    #[error("No invitation found for provided token.")]
    InvitationNotFound,
    #[error("Invitation accept token has expired.")]
    ExpiredAcceptToken,
    #[error("{0}")]
    ServiceError(#[from] invitation_service::InvitationAcceptError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl InvitationController {
    pub async fn reject(
        db: &DatabaseConnection,
        token: InvitationToken,
    ) -> Result<(), InvitationRejectError> {
        // NOTE: We don't check that the invitation status is "SENT"
        //   because it would cause a lot of useless edge cases.
        let invitation = Self::get_by_reject_token(db, token.clone())
            .await?
            .ok_or(InvitationRejectError::InvitationNotFound)?;
        // NOTE: An extra layer of security *just in case*
        assert_eq!(*token.expose_secret(), invitation.reject_token);

        invitation.delete(db).await?;

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
    pub async fn resend<'r>(
        db: &DatabaseConnection,
        config: &AppConfig,
        notifier: &Notifier<'r>,
        invitation_id: i32,
    ) -> Result<(), InvitationResendError> {
        let invitation = InvitationRepository::get_by_id(db, &invitation_id)
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
    pub async fn cancel<'r>(
        db: &DatabaseConnection,
        invitation_id: i32,
    ) -> Result<(), InvitationCancelError> {
        InvitationRepository::delete_by_id(db, invitation_id).await?;

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
        db: &DatabaseConnection,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), DbErr> {
        InvitationRepository::get_all(db, page_number, page_size, until).await
    }
}
