// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::sync::Arc;

    pub use anyhow::Context as _;
    pub use async_trait::async_trait;
    pub use chrono::TimeDelta;

    pub use crate::{
        auth::{AuthService, AuthToken, Password},
        errors::Forbidden,
        invitations::{
            errors::*,
            invitation_repository::{CreateAccountInvitationCommand, InvitationRepository},
            models::*,
        },
        licensing::LicensingService,
        members::{Member, MemberRole, Nickname, UserRepository},
        models::{DatabaseRwConnectionPools, EmailAddress},
        notifications::{notifier::email::EmailNotification, NotificationService},
        onboarding,
        prosody::prosody_invites_register_api,
        util::{either::*, JidExt as _},
        workspace::WorkspaceService,
        xmpp::{
            jid::{BareJid, JidNode},
            XmppService, XmppServiceContext,
        },
        AppConfig,
    };

    pub use super::{
        AcceptAccountInvitationCommand, InvitationApplicationServiceImpl, InviteUserCommand,
        RegisterResponse,
    };
}

use crate::licensing::errors::UserLimitReached;

pub use self::live_invitation_service::LiveInvitationApplicationService;
use self::prelude::*;

#[derive(Debug)]
pub struct InvitationService {
    pub db: DatabaseRwConnectionPools,
    pub notification_service: NotificationService,
    pub invitation_repository: InvitationRepository,
    pub workspace_service: WorkspaceService,
    pub auth_service: AuthService,
    pub xmpp_service: XmppService,
    pub user_repository: UserRepository,
    pub app_config: Arc<AppConfig>,
    pub invitation_application_service: InvitationApplicationService,
    pub licensing_service: LicensingService,
}

impl InvitationService {
    pub async fn invite_user(
        &self,
        command: InviteUserCommand,
        auth: &AuthToken,
    ) -> Result<Invitation, InviteUserError> {
        let ref username = command.username;
        let email_address = command.email_address.clone();

        // Test if an invitation already exists for the given username.
        if self
            .invitation_repository
            .get_account_invitation_by_username(username, auth)
            .await?
            .is_some()
        {
            return Err(InvitationAlreadyExists.into());
        }

        // Test if a user already exists with the given username.
        if self.user_repository.user_exists(username, auth).await? {
            return Err(UsernameAlreadyTaken.into());
        }

        // Create the invitation on the Server.
        let invitation = self
            .invitation_repository
            .create_account_invitation(command.into(), auth)
            .await?;

        // Store that at least one invitation has been sent.
        (onboarding::at_least_one_invitation_sent::set(&self.db.write, true).await)
            .inspect_err(|err| {
                tracing::warn!("Could not set `at_least_one_invitation_sent` to true: {err}")
            })
            .ok();

        // Send the notification.
        self.send_account_invitation_notification(&invitation, email_address, auth)
            .await?;

        Ok(invitation)
    }

    pub async fn accept_account_invitation(
        &self,
        token: InvitationToken,
        command: AcceptAccountInvitationCommand,
    ) -> Result<
        Member,
        Either4<UserLimitReached, InvitationNotFoundForToken, MemberAlreadyExists, anyhow::Error>,
    > {
        // Check user limit.
        let user_count = self.user_repository.users_stats(None).await?.count as u32;
        if !self.licensing_service.allows_user_count(user_count + 1) {
            return Err(Either4::E1(UserLimitReached));
        }

        let Some(invitation) = self
            .invitation_repository
            .get_account_invitation_by_token(&token)
            .await?
        else {
            return Err(Either4::E2(InvitationNotFoundForToken));
        };

        let jid = self
            .invitation_application_service
            .register_with_token(&command.password, token)
            .await?
            .jid;

        // Log user in.
        // NOTE: We need to log the user in to get a Prosody
        //   authentication token in order to set the user’s vCard.
        let auth_token = (self.auth_service)
            .log_in(&jid, &command.password)
            .await
            .expect("User credentials should work after creating an account");

        // Creates the user’s vCard.
        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            auth_token: auth_token.clone(),
        };
        let email_address = command.email.unwrap_or(invitation.email_address);
        self.xmpp_service
            .create_own_vcard(&ctx, &command.nickname, Some(email_address))
            .await
            .context("Could not create user vCard4")
            .map_err(Either4::E4)?;

        let user_info = self
            .auth_service
            .get_user_info(&auth_token)
            .await
            .map_err(anyhow::Error::new)
            .context("Could not get own account info")?;
        let member = Member::from(user_info);

        // Revoke token because it will never be used again.
        self.auth_service
            .revoke(ctx.auth_token)
            .await
            .context("Could not revoke temporary auth token")
            .map_err(Either4::E4)?;

        Ok(member)
    }

    pub async fn reject_account_invitation(
        &self,
        token: InvitationToken,
    ) -> Result<(), anyhow::Error> {
        self.invitation_application_service
            .reject_invitation(token)
            .await?;
        Ok(())
    }

    pub async fn resend_account_invitation(
        &self,
        invitation_id: &InvitationId,
        auth: &AuthToken,
    ) -> Result<(), Either3<Forbidden, InvitationNotFound, anyhow::Error>> {
        let invitation = self
            .invitation_repository
            .get_account_invitation_by_id(&invitation_id, auth)
            .await
            .map_err(to_either3_1_3)?
            .ok_or(Either3::E2(InvitationNotFound(invitation_id.clone())))?;

        let email_address = invitation.email_address.clone();

        // Send the notification.
        self.send_account_invitation_notification(&invitation, email_address, auth)
            .await?;

        Ok(())
    }

    pub async fn cancel_invitation(
        &self,
        invitation_id: InvitationId,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        self.invitation_repository
            .delete_invitation(invitation_id, auth)
            .await
    }

    async fn send_account_invitation_notification(
        &self,
        invitation: &Invitation,
        email_address: EmailAddress,
        auth: &AuthToken,
    ) -> Result<(), anyhow::Error> {
        // Construct the notification payload.
        let payload = {
            let workspace_name = (self.workspace_service)
                .get_workspace_name(Some(auth))
                .await
                .context("Could not get workspace details (to build the notification)")?;

            WorkspaceInvitationPayload {
                accept_token: invitation.accept_token.clone(),
                reject_token: invitation.reject_token.clone(),
                workspace_name,
                dashboard_url: self.app_config.dashboard_url().clone(),
                api_app_name: self.app_config.branding.api_app_name.clone(),
                organization_name: self.app_config.branding.company_name.clone(),
            }
        };

        // Create the notification.
        let notification =
            EmailNotification::for_workspace_invitation(email_address, payload, &self.app_config)
                .context("Could not create email")?;

        // Send the notification.
        self.notification_service
            .send_email(notification)
            .context("Could not send account invitation: {err}")?;

        Ok(())
    }
}

pub struct InviteUserCommand {
    pub username: JidNode,
    pub role: MemberRole,
    pub email_address: EmailAddress,
    pub ttl: Option<TimeDelta>,
}

#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("{0}")]
pub enum InviteUserError {
    Forbidden(#[from] Forbidden),
    InvitationAlreadyExists(#[from] InvitationAlreadyExists),
    UsernameAlreadyTaken(#[from] UsernameAlreadyTaken),
    Internal(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct AcceptAccountInvitationCommand {
    pub nickname: Nickname,
    pub password: Password,
    pub email: Option<EmailAddress>,
}

// MARK: - Application Service

/// [`InvitationService`] has domain logic only, but some actions
/// still need to be mockable and don’t belong in [`InvitationRepository`].
/// This is where those functions go.
#[derive(Debug, Clone)]
pub struct InvitationApplicationService {
    pub implem: Arc<dyn InvitationApplicationServiceImpl>,
}

#[async_trait]
pub trait InvitationApplicationServiceImpl: std::fmt::Debug + Sync + Send {
    async fn register_with_token(
        &self,
        password: &Password,
        token: InvitationToken,
    ) -> Result<
        RegisterResponse,
        Either3<InvitationNotFoundForToken, MemberAlreadyExists, anyhow::Error>,
    >;

    async fn reject_invitation(&self, token: InvitationToken) -> Result<(), anyhow::Error>;
}

pub struct RegisterResponse {
    pub jid: BareJid,
}

mod live_invitation_service {
    use crate::prosody::ProsodyInvitesRegisterApi;

    use super::*;

    #[derive(Debug)]
    pub struct LiveInvitationApplicationService {
        pub invites_register_api: ProsodyInvitesRegisterApi,
    }

    #[async_trait]
    impl InvitationApplicationServiceImpl for LiveInvitationApplicationService {
        async fn register_with_token(
            &self,
            password: &Password,
            token: InvitationToken,
        ) -> Result<
            RegisterResponse,
            Either3<InvitationNotFoundForToken, MemberAlreadyExists, anyhow::Error>,
        > {
            match self
                .invites_register_api
                .register_with_invite(None, password, token)
                .await
            {
                Ok(res) => Ok(RegisterResponse::from(res)),
                Err(Either4::E1(err)) => Err(Either3::E1(err)),
                Err(Either4::E3(err)) => Err(Either3::E2(err)),
                // NOTE: `403 Forbidden`s can technically happen, but it’d mean
                //   something is not configured properly internally.
                Err(Either4::E2(err @ Forbidden(_))) => Err(Either3::E3(anyhow::Error::new(err))),
                Err(Either4::E4(err)) => Err(Either3::E3(err)),
            }
        }

        async fn reject_invitation(&self, token: InvitationToken) -> Result<(), anyhow::Error> {
            self.invites_register_api.reject_invite(token).await
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for InvitationApplicationService {
    type Target = Arc<dyn InvitationApplicationServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

impl<E1, E2> From<Either<E1, E2>> for InviteUserError
where
    InviteUserError: From<E1> + From<E2>,
{
    fn from(either: Either<E1, E2>) -> Self {
        match either {
            Either::E1(err) => Self::from(err),
            Either::E2(err) => Self::from(err),
        }
    }
}

impl From<InviteUserCommand> for CreateAccountInvitationCommand {
    fn from(command: InviteUserCommand) -> Self {
        Self {
            username: command.username,
            role: command.role,
            email_address: command.email_address,
            ttl: command.ttl,
        }
    }
}

impl From<prosody_invites_register_api::RegisterResponse> for RegisterResponse {
    fn from(response: prosody_invites_register_api::RegisterResponse) -> Self {
        Self { jid: response.jid }
    }
}
