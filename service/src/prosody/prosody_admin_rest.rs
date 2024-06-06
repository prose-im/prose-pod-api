// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Write as _, path::PathBuf};

use entity::{
    model::{MemberRole, JID},
    server_config,
};
use log::debug;
use reqwest::{Client, RequestBuilder, Response};
use tokio::runtime::Handle;

use crate::{config::Config, server_ctl::*};

use super::{prosody_config_from_db, AsProsody as _};

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug)]
pub struct ProsodyAdminRest {
    config_file_path: PathBuf,
    admin_rest_api_url: String,
    api_auth_username: JID,
    api_auth_password: String,
}

impl ProsodyAdminRest {
    pub fn from_config(config: &Config) -> Self {
        Self {
            config_file_path: config.server.prosody_config_file_path.to_owned(),
            admin_rest_api_url: config.server.admin_rest_api_url(),
            api_auth_username: config.api_jid(),
            api_auth_password: config.api.admin_password.to_owned().unwrap(),
        }
    }

    fn call(&self, make_req: impl FnOnce(&Client) -> RequestBuilder) -> Result<Response, Error> {
        let client = Client::new();
        let request = make_req(&client)
            .basic_auth(
                self.api_auth_username.to_string(),
                Some(self.api_auth_password.clone()),
            )
            .build()?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let response = client.execute(request).await?;
                if response.status().is_success() {
                    Ok(response)
                } else {
                    Err(Error::Other(format!(
                        "Prosody Admin REST API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                        response.status(),
                        response.headers().clone(),
                        response.text().await.unwrap_or("<nil>".to_string())
                    )))
                }
            })
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_url)
    }
}

impl ServerCtlImpl for ProsodyAdminRest {
    fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error> {
        let mut file = File::create(&self.config_file_path)?;
        file.write_all(
            prosody_config_from_db(server_config.to_owned(), app_config)
                .to_string()
                .as_bytes(),
        )?;
        Ok(())
    }
    fn reload(&self) -> Result<(), Error> {
        self.call(|client| client.put(self.url("reload")))
            .map(|_| ())
    }

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error> {
        self.call(|client| {
            client
                .post(format!(
                    "{}/{}",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string())
                ))
                .body(format!(r#"{{"password":"{}"}}"#, password))
        })
        .map(|_| ())
    }
    fn remove_user(&self, jid: &JID) -> Result<(), Error> {
        self.call(|client| {
            client.delete(format!(
                "{}/{}",
                self.url("user"),
                urlencoding::encode(&jid.to_string())
            ))
        })
        .map(|_| ())
    }
    fn set_user_role(&self, jid: &JID, role: &MemberRole) -> Result<(), Error> {
        self.call(|client| {
            client
                .patch(format!(
                    "{}/{}/role",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string()),
                ))
                .body(format!(r#"{{"role":"{}"}}"#, role.as_prosody()))
        })
        .map(|_| ())
    }
}
