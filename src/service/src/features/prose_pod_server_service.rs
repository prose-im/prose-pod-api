// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    auth::AuthToken, errors::*, invitations::models::*, members::MemberRole,
    prosody::prosody_http_admin_api::InviteInfo, util::either::*, AppConfig, ServerConfig,
};

pub use self::live_prose_pod_server_service::LiveProsePodServerService;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ProsePodServerService(pub Arc<dyn ProsePodServerServiceImpl>);

impl std::ops::Deref for ProsePodServerService {
    type Target = Arc<dyn ProsePodServerServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
pub trait ProsePodServerServiceImpl: std::fmt::Debug + Sync + Send {
    async fn wait_until_ready(&self) -> Result<(), anyhow::Error>;

    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
        auth: Option<&AuthToken>,
    ) -> Result<(), anyhow::Error>;

    async fn reset_config(&self, auth: &AuthToken) -> Result<(), Either<Forbidden, anyhow::Error>>;

    async fn reload(&self, auth: Option<&AuthToken>) -> Result<(), anyhow::Error>;

    async fn delete_all_data(&self, auth: &AuthToken) -> Result<(), anyhow::Error>;
}

mod live_prose_pod_server_service {
    use std::path::PathBuf;

    use anyhow::Context as _;

    use crate::{
        auth::AuthService,
        models::DatabaseRwConnectionPools,
        prose_pod_server_api::ProsePodServerApi,
        prosody::{
            ProsodyAdminRest, ProsodyHttpAdminApi, ProsodyInvitesRegisterApi, ProsodyOAuth2,
        },
        xmpp::XmppService,
    };

    use super::*;

    #[derive(Debug, Clone)]
    pub struct LiveProsePodServerService {
        pub config_file_path: PathBuf,
        pub server_api: ProsePodServerApi,
        pub admin_rest: Arc<ProsodyAdminRest>,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
        pub auth_service: AuthService,
        pub xmpp_service: XmppService,
        pub oauth2: Arc<ProsodyOAuth2>,
        pub invites_register_api: ProsodyInvitesRegisterApi,
        pub db: DatabaseRwConnectionPools,
    }

    #[async_trait]
    impl ProsePodServerServiceImpl for LiveProsePodServerService {
        #[tracing::instrument(
            name = "pod::server::wait_until_ready",
            level = "trace",
            skip_all,
            err
        )]
        async fn wait_until_ready(&self) -> Result<(), anyhow::Error> {
            use std::convert::identity as is_true;
            use std::time::{Duration, Instant};
            use tokio::time::sleep;

            let start = Instant::now();
            let timeout = Duration::from_secs(10);
            let retry_interval = Duration::from_millis(100);

            while !self.server_api.is_healthy().await.is_ok_and(is_true)
                && start.elapsed() < timeout
            {
                sleep(retry_interval).await;
            }

            if start.elapsed() >= timeout {
                return Err(anyhow::Error::msg(
                    "Timed out while waiting for the Server. Check Server logs.",
                ));
            }

            Ok(())
        }

        #[tracing::instrument(name = "pod::server::save_config", level = "trace", skip_all, err)]
        async fn save_config(
            &self,
            server_config: &ServerConfig,
            app_config: &AppConfig,
            auth: Option<&AuthToken>,
        ) -> Result<(), anyhow::Error> {
            use crate::prosody_config_from_db;
            use std::fs::File;
            use std::io::Write as _;

            let mut file = File::create(&self.config_file_path).context(format!(
                "Cannot create Prosody config file at path `{path}`",
                path = self.config_file_path.display()
            ))?;

            let admins = self.server_api.users_util_admin_jids(auth).await?;

            let prosody_config = prosody_config_from_db(
                &self.db.read,
                app_config,
                Some(server_config.to_owned()),
                admins,
            )
            .await?;
            file.write_all(
                prosody_config
                    .to_string(server_config.prosody_overrides_raw.as_deref().cloned())
                    .as_bytes(),
            )
            .context(format!(
                "Cannot write Prosody config file at path `{path}`",
                path = self.config_file_path.display()
            ))?;

            Ok(())
        }

        #[tracing::instrument(name = "pod::server::reset_config", level = "trace", skip_all, err)]
        async fn reset_config(
            &self,
            // NOTE: Not used but here as last-resort a safety guard.
            _auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            use crate::prosody::prosody_bootstrap_config;
            use std::fs::File;
            use std::io::Write as _;

            let mut file = File::create(&self.config_file_path).context(format!(
                "Cannot create Prosody config file at path `{path}`",
                path = self.config_file_path.display()
            ))?;

            let prosody_config = prosody_bootstrap_config();
            let prosody_config_file = prosody_config.print_with_bootstrap_header();
            file.write_all(prosody_config_file.to_string().as_bytes())
                .context(format!(
                    "Cannot write Prosody config file at path `{path}`",
                    path = self.config_file_path.display()
                ))?;

            Ok(())
        }

        #[tracing::instrument(name = "pod::server::reload", level = "trace", skip_all, err)]
        async fn reload(&self, auth: Option<&AuthToken>) -> Result<(), anyhow::Error> {
            self.server_api.lifecycle_reload(auth).await?;
            Ok(())
        }

        #[tracing::instrument(
            name = "pod::server::delete_all_data",
            level = "trace",
            skip_all,
            err
        )]
        async fn delete_all_data(&self, auth: &AuthToken) -> Result<(), anyhow::Error> {
            let todo = "Move to Server API";

            self.admin_rest
                .call(|client| client.delete(self.admin_rest.url("certs")), auth)
                .await?;
            // NOTE: Delete data last otherwise API calls fail because of authentication.
            self.admin_rest
                .call(|client| client.delete(self.admin_rest.url("data")), auth)
                .await?;

            Ok(())
        }
    }

    // MARK: - Boilerplate

    impl TryFrom<InviteInfo> for Invitation {
        type Error = anyhow::Error;

        fn try_from(invite: InviteInfo) -> Result<Self, Self::Error> {
            use crate::models::EmailAddress;
            use anyhow::anyhow;
            use std::str::FromStr as _;

            let pre_assigned_role = invite
                .roles
                .iter()
                .flat_map(|s| MemberRole::from_str(s))
                .next()
                .unwrap_or_default();

            let Some(email_address) = invite.additional_data.get("email") else {
                // NOTE: Until we implement #342, this should have been set already.
                return Err(anyhow!("Email address not stored in the invite additional data. Invite might have been created outside of Prose, which is unsupported."));
            };
            let email_address: EmailAddress = serde_json::from_value(email_address.clone())
                .context("Email address in invitation is invalid")?;

            Ok(Self {
                id: invite.id.clone(),
                created_at: invite.created_at,
                jid: invite.jid,
                pre_assigned_role,
                email_address: email_address,
                accept_token: invite.id.clone(),
                accept_token_expires_at: invite.expires,
                reject_token: invite.id.clone(),
            })
        }
    }
}
