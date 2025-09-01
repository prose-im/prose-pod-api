// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Write as _, path::PathBuf, sync::Arc};

use reqwest::Method;
use secrecy::{ExposeSecret as _, SecretString};
use tokio::time::{sleep, Duration, Instant};
use tracing::instrument;

use crate::{
    members::MemberRole,
    prosody::{
        prosody_admin_rest, prosody_bootstrap_config, prosody_config_from_db, AsProsody as _,
        ProsodyAdminRest,
    },
    xmpp::{server_ctl, BareJid, ServerCtlImpl},
    AppConfig, ServerConfig,
};

#[derive(Debug, Clone)]
pub struct LiveServerCtl {
    config_file_path: PathBuf,
    admin_rest: Arc<ProsodyAdminRest>,
}

impl LiveServerCtl {
    pub fn from_config(config: &AppConfig, admin_rest: Arc<ProsodyAdminRest>) -> Self {
        Self {
            config_file_path: config.prosody_ext.config_file_path.to_owned(),
            admin_rest,
        }
    }
}

#[async_trait::async_trait]
impl ServerCtlImpl for LiveServerCtl {
    #[instrument(name = "server_ctl::wait_until_ready", level = "trace", skip_all, err)]
    async fn wait_until_ready(&self) -> Result<(), server_ctl::Error> {
        let start = Instant::now();
        let timeout = Duration::from_secs(10);
        let retry_interval = Duration::from_millis(100);

        while self
            .admin_rest
            .call(|client| client.get(self.admin_rest.url("modules")))
            .await
            .is_err()
            && start.elapsed() < timeout
        {
            sleep(retry_interval).await;
        }

        if start.elapsed() >= timeout {
            return Err(server_ctl::Error::Other("Timed out while waiting for the XMPP server. Possible causes (not exhaustive): [`mod_admin_rest`](https://github.com/RemiBardon/prosody-mod_admin_rest) module not enabled, HTTP interface unreachable.".to_string()));
        }

        Ok(())
    }

    #[instrument(name = "server_ctl::save_config", level = "trace", skip_all, err)]
    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
    ) -> Result<(), server_ctl::Error> {
        let mut file = File::create(&self.config_file_path).map_err(|e| {
            server_ctl::Error::CannotOpenConfigFile(self.config_file_path.clone(), e)
        })?;
        let prosody_config = prosody_config_from_db(server_config.to_owned(), app_config);
        file.write_all(
            prosody_config
                .to_string(server_config.prosody_overrides_raw.as_deref().cloned())
                .as_bytes(),
        )
        .map_err(|e| server_ctl::Error::CannotWriteConfigFile(self.config_file_path.clone(), e))?;

        Ok(())
    }
    #[instrument(name = "server_ctl::reset_config", level = "trace", skip_all, err)]
    async fn reset_config(
        &self,
        init_admin_password: &SecretString,
    ) -> Result<(), server_ctl::Error> {
        let mut file = File::create(&self.config_file_path).map_err(|e| {
            server_ctl::Error::CannotOpenConfigFile(self.config_file_path.clone(), e)
        })?;

        let prosody_config = prosody_bootstrap_config(init_admin_password);
        let prosody_config_file = prosody_config.print_with_bootstrap_header();
        file.write_all(prosody_config_file.to_string().as_bytes())
            .map_err(|e| {
                server_ctl::Error::CannotWriteConfigFile(self.config_file_path.clone(), e)
            })?;

        Ok(())
    }
    #[instrument(name = "server_ctl::reload", level = "trace", skip_all, err)]
    async fn reload(&self) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| client.put(self.admin_rest.url("reload")))
            .await
            .map(|_| ())
    }

    #[instrument(name = "server_ctl::list_users", level = "trace", skip_all, err)]
    async fn list_users(&self) -> Result<Vec<server_ctl::User>, server_ctl::Error> {
        let response = self.admin_rest.list_users().await?;
        Ok(response.into_iter().map(server_ctl::User::from).collect())
    }
    #[instrument(
        name = "server_ctl::add_user", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    async fn add_user(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| {
                client
                    .post(format!(
                        "{}/{}",
                        self.admin_rest.url("user"),
                        urlencoding::encode(&jid.to_string())
                    ))
                    .body(format!(r#"{{"password":"{}"}}"#, password.expose_secret()))
            })
            .await?;

        Ok(())
    }
    #[instrument(
        name = "server_ctl::remove_user", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    async fn remove_user(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| {
                client.delete(format!(
                    "{}/{}",
                    self.admin_rest.url("user"),
                    urlencoding::encode(&jid.to_string())
                ))
            })
            .await?;

        Ok(())
    }

    #[instrument(
        name = "server_ctl::set_user_role", level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            role = role.to_string(),
        ),
        err,
    )]
    async fn set_user_role(
        &self,
        jid: &BareJid,
        role: &MemberRole,
    ) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| {
                client
                    .patch(format!(
                        "{}/{}/role",
                        self.admin_rest.url("user"),
                        urlencoding::encode(&jid.to_string()),
                    ))
                    .body(format!(r#"{{"role":"{}"}}"#, role.as_prosody()))
            })
            .await
            .map(|_| ())
    }
    #[instrument(
        name = "server_ctl::set_user_password", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    async fn set_user_password(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| {
                client
                    .patch(format!(
                        "{}/{}/password",
                        self.admin_rest.url("user"),
                        urlencoding::encode(&jid.to_string())
                    ))
                    .body(format!(r#"{{"password":"{}"}}"#, password.expose_secret()))
            })
            .await
            .map(|_| ())
    }

    #[instrument(
        name = "server_ctl::add_team_member", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    async fn add_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .update_team_members(Method::PUT, jid)
            .await?;
        Ok(())
    }
    #[instrument(
        name = "server_ctl::remove_team_member", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    async fn remove_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .update_team_members(Method::DELETE, jid)
            .await?;
        Ok(())
    }
    async fn force_rosters_sync(&self) -> Result<(), server_ctl::Error> {
        self.admin_rest.update_rosters().await
    }

    #[instrument(name = "server_ctl::delete_all_data", level = "trace", skip_all, err)]
    async fn delete_all_data(&self) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| client.delete(self.admin_rest.url("certs")))
            .await?;
        // NOTE: Delete data last otherwise API calls fail because of authentication.
        self.admin_rest
            .call(|client| client.delete(self.admin_rest.url("data")))
            .await?;
        Ok(())
    }
}

impl From<prosody_admin_rest::ListUsersItem> for server_ctl::User {
    fn from(value: prosody_admin_rest::ListUsersItem) -> Self {
        Self {
            jid: value.jid,
            role: value.role.into(),
        }
    }
}

impl From<prosody_admin_rest::Role> for server_ctl::Role {
    fn from(value: prosody_admin_rest::Role) -> Self {
        Self {
            name: value.name,
            inherits: value
                .inherits
                .into_iter()
                .map(server_ctl::Role::from)
                .collect(),
        }
    }
}
