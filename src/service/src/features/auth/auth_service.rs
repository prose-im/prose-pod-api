// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::{sync::Arc, time::Duration};

    pub use async_trait::async_trait;
    pub use secrecy::SecretString;

    pub use crate::{
        auth::{
            auth_service::{self, AuthServiceImpl},
            errors::{InvalidCredentials, PasswordResetTokenExpired, PasswordValidationError},
            models::{AuthToken, Password, PasswordResetRequestInfo, PasswordResetToken, UserInfo},
        },
        errors::{Forbidden, Unauthorized},
        invitations::InvitationContact,
        models::jid::{BareJid, NodeRef},
        util::either::{Either, Either3, Either4},
    };
}

pub use self::live_auth_service::LiveAuthService;
use self::prelude::*;

#[derive(Debug, Clone)]
pub struct AuthService {
    pub implem: Arc<dyn AuthServiceImpl>,
}

#[inline]
pub fn validate_password(password: &Password, min_len: u8) -> Result<(), PasswordValidationError> {
    use secrecy::ExposeSecret as _;

    let len = password.expose_secret().len();
    if len < min_len as usize {
        return Err(PasswordValidationError::TooShort { min_len, len });
    };

    Ok(())
}

#[async_trait::async_trait]
pub trait AuthServiceImpl: std::fmt::Debug + Sync + Send {
    async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>>;

    async fn get_user_info(
        &self,
        auth: &AuthToken,
    ) -> Result<UserInfo, Either<Unauthorized, anyhow::Error>>;

    async fn revoke(&self, token: AuthToken) -> Result<(), anyhow::Error>;

    async fn create_password_reset_token(
        &self,
        username: &NodeRef,
        ttl: Option<Duration>,
        contact: &InvitationContact,
        auth: &AuthToken,
    ) -> Result<PasswordResetRequestInfo, Either<Forbidden, anyhow::Error>>;

    fn validate_password(&self, password: &Password) -> Result<(), PasswordValidationError>;

    async fn reset_password(
        &self,
        token: PasswordResetToken,
        password: &Password,
    ) -> Result<
        (),
        Either4<PasswordValidationError, PasswordResetTokenExpired, Forbidden, anyhow::Error>,
    >;
}

mod live_auth_service {
    use anyhow::{anyhow, Context as _};

    use crate::{
        invitations::errors::{InvitationNotFoundForToken, MemberAlreadyExists},
        prose_pod_server_api::{ProsePodServerApi, ProsePodServerError},
        prosody::{
            prosody_http_admin_api::InviteInfo, ProsodyHttpAdminApi, ProsodyInvitesRegisterApi,
            ProsodyOAuth2, ProsodyOAuth2Error,
        },
        util::either::Either4,
        xmpp::JidNode,
    };

    use super::*;

    #[derive(Debug)]
    pub struct LiveAuthService {
        pub oauth2: Arc<ProsodyOAuth2>,
        pub server_api: ProsePodServerApi,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
        pub invites_register_api: ProsodyInvitesRegisterApi,
        pub password_reset_token_ttl: Duration,
        pub min_password_length: u8,
    }

    #[async_trait::async_trait]
    impl AuthServiceImpl for LiveAuthService {
        async fn log_in(
            &self,
            jid: &BareJid,
            password: &SecretString,
        ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
            match self.oauth2.log_in(jid, password).await {
                Ok(Some(token)) => Ok(AuthToken(token)),
                Ok(None) => Err(Either::E1(InvalidCredentials)),
                Err(ProsodyOAuth2Error::Unauthorized(_)) => Err(Either::E1(InvalidCredentials)),
                Err(err) => Err(Either::E2(anyhow!(err).context("Prosody OAuth 2.0 error"))),
            }
        }

        async fn get_user_info(
            &self,
            auth: &AuthToken,
        ) -> Result<UserInfo, Either<Unauthorized, anyhow::Error>> {
            match self.server_api.users_util_self(auth).await {
                Ok(user_info) => Ok(user_info),
                Err(ProsePodServerError::Forbidden(Forbidden(msg))) => {
                    Err(Either::E1(Unauthorized(msg)))
                }
                Err(err) => Err(Either::E2(anyhow::Error::new(err))),
            }
        }

        async fn revoke(&self, token: AuthToken) -> Result<(), anyhow::Error> {
            self.oauth2
                .revoke(token)
                .await
                .context("Prosody OAuth 2.0 error")
        }

        async fn create_password_reset_token(
            &self,
            username: &NodeRef,
            ttl: Option<Duration>,
            contact: &InvitationContact,
            auth: &AuthToken,
        ) -> Result<PasswordResetRequestInfo, Either<Forbidden, anyhow::Error>> {
            use crate::auth::PasswordResetToken;
            use crate::prosody::prosody_http_admin_api::CreateAccountResetInvitationRequest;
            use serde_json::json;

            let email_address = match contact {
                InvitationContact::Email { email_address } => email_address,
            };

            // Read token TTL from app config.
            // NOTE: We can’t just put the `AppConfig` value in the Prosody
            //   configuration, as `mod_invites` has a special `ttl or 86400`
            //   and via `mod_http_admin_api` `ttl` comes from the request
            //   body exclusively.
            let default_ttl_secs = self.password_reset_token_ttl.as_secs();

            // Create the password reset token.
            let invite: InviteInfo = self
                .admin_api
                .create_invite_for_account_reset(
                    CreateAccountResetInvitationRequest {
                        username: Some(JidNode::from(username)),
                        ttl_secs: Some(ttl.map_or(default_ttl_secs, |d| d.as_secs()) as u32),
                        additional_data: json!({
                            "email": email_address,
                        }),
                    },
                    auth,
                )
                .await?;

            Ok(PasswordResetRequestInfo {
                jid: invite.jid,
                token: PasswordResetToken::from(invite.id),
                created_at: invite.created_at,
                expires_at: invite.expires,
            })
        }

        fn validate_password(&self, password: &Password) -> Result<(), PasswordValidationError> {
            auth_service::validate_password(password, self.min_password_length)
        }

        async fn reset_password(
            &self,
            token: PasswordResetToken,
            password: &Password,
        ) -> Result<
            (),
            Either4<PasswordValidationError, PasswordResetTokenExpired, Forbidden, anyhow::Error>,
        > {
            self.validate_password(password).map_err(Either4::E1)?;

            match self
                .invites_register_api
                .register_with_invite(None, password, token)
                .await
            {
                Ok(_) => Ok(()),
                Err(Either4::E1(InvitationNotFoundForToken)) => {
                    Err(Either4::E2(PasswordResetTokenExpired))
                }
                Err(Either4::E2(err)) => Err(Either4::E3(err)),
                Err(Either4::E3(MemberAlreadyExists(_))) => unreachable!(),
                Err(Either4::E4(err)) => Err(Either4::E4(err)),
            }
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for AuthService {
    type Target = Arc<dyn AuthServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
