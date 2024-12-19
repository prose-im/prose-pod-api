// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Write as _, path::PathBuf, sync::Arc};

use reqwest::Method;
use secrecy::{ExposeSecret as _, SecretString};
use tokio::time::{sleep, Duration, Instant};
use tracing::error;

use crate::{
    members::MemberRole,
    prosody::{prosody_config_from_db, AsProsody as _, ProsodyAdminRest},
    server_config::ServerConfig,
    xmpp::{server_ctl, BareJid, ServerCtlImpl},
    AppConfig,
};

#[derive(Debug, Clone)]
pub struct LiveServerCtl {
    config_file_path: PathBuf,
    admin_rest: Arc<ProsodyAdminRest>,
}

impl LiveServerCtl {
    pub fn from_config(config: &AppConfig, admin_rest: Arc<ProsodyAdminRest>) -> Self {
        Self {
            config_file_path: config.server.prosody_config_file_path.to_owned(),
            admin_rest,
        }
    }
}

#[async_trait::async_trait]
impl ServerCtlImpl for LiveServerCtl {
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
            error!("Timed out while waiting for the XMPP server. You probably forgot to enable the [`mod_admin_rest`](https://github.com/RemiBardon/prosody-mod_admin_rest) module.");
        }

        Ok(())
    }

    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
    ) -> Result<(), server_ctl::Error> {
        let mut file = File::create(&self.config_file_path).map_err(|e| {
            server_ctl::Error::CannotOpenConfigFile(self.config_file_path.clone(), e)
        })?;
        let prosody_config = prosody_config_from_db(server_config.to_owned(), app_config);
        file.write_all(prosody_config.to_string().as_bytes())
            .map_err(|e| {
                server_ctl::Error::CannotWriteConfigFile(self.config_file_path.clone(), e)
            })?;

        Ok(())
    }
    async fn reload(&self) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .call(|client| client.put(self.admin_rest.url("reload")))
            .await
            .map(|_| ())
    }

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

    async fn add_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .update_team_members(Method::PUT, jid)
            .await?;
        Ok(())
    }
    async fn remove_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.admin_rest
            .update_team_members(Method::DELETE, jid)
            .await?;
        Ok(())
    }
}
