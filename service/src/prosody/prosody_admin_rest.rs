// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, future::Future, io::Write as _, path::PathBuf};

use crate::{entity::server_config, model::MemberRole};
use prose_xmpp::BareJid;
use reqwest::{
    header::HeaderMap, Client as HttpClient, Method, RequestBuilder, Response, StatusCode,
};
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use tracing::debug;

use crate::{
    config::Config,
    services::{
        live_xmpp_service::NonStandardXmppClient,
        server_ctl::{Error, ServerCtlImpl},
    },
};

use super::{prosody_config_from_db, AsProsody as _};

const TEAM_GROUP_ID: &'static str = "team";
const TEAM_GROUP_NAME: &'static str = "Team";

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug, Clone)]
pub struct ProsodyAdminRest {
    http_client: HttpClient,
    config_file_path: PathBuf,
    admin_rest_api_url: String,
    admin_rest_api_on_main_host_url: String,
    api_auth_username: BareJid,
    api_auth_password: SecretString,
}

impl ProsodyAdminRest {
    pub fn from_config(config: &Config, http_client: HttpClient) -> Self {
        Self {
            http_client,
            config_file_path: config.server.prosody_config_file_path.to_owned(),
            admin_rest_api_url: config.server.admin_rest_api_url(),
            admin_rest_api_on_main_host_url: config.server.admin_rest_api_on_main_host_url(),
            api_auth_username: config.api_jid(),
            api_auth_password: config.api.admin_password.to_owned().unwrap(),
        }
    }

    async fn call(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
    ) -> Result<Response, Error> {
        self.call_(make_req, |response| async {
            if response.status().is_success() {
                Ok(response)
            } else {
                Err(ResponseData {
                    status: response.status(),
                    headers: response.headers().clone(),
                    body: response.text().await.unwrap_or("<nil>".into()),
                })
            }
        })
        .await
    }

    async fn call_<T, F: Future<Output = Result<T, ResponseData>>>(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        map_res: impl FnOnce(Response) -> F,
    ) -> Result<T, Error> {
        let client = self.http_client.clone();
        let request = make_req(&client)
            .basic_auth(
                self.api_auth_username.to_string(),
                Some(self.api_auth_password.expose_secret()),
            )
            .build()?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        let (response, request_clone) = {
            let request_clone = request.try_clone();
            (
                client.execute(request).await.map_err(|err| {
                    Error::Other(format!("Prosody Admin REST API call failed: {err}"))
                })?,
                request_clone,
            )
        };
        match map_res(response).await {
            Ok(res) => Ok(res),
            Err(response) => {
                let mut err = "Prosody Admin REST API call failed.".to_owned();
                if let Some(request) = request_clone {
                    err.push_str(&format!(
                        "\n  Request: {} {}\n  Request headers: {:?}\n  Request body: {:?}",
                        request.method(),
                        request.url().to_string(),
                        request.headers().clone(),
                        request
                            .body()
                            .and_then(|body| body.as_bytes())
                            .map(std::str::from_utf8),
                    ));
                }
                err.push_str(&format!(
                    "\n  Response status: {}\n  Response headers: {:?}\n  Response body: {}",
                    response.status, response.headers, response.body,
                ));
                Err(Error::Other(err))
            }
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_url)
    }
    fn url_on_main_host(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_on_main_host_url)
    }

    async fn create_team_group(&self) -> Result<(), Error> {
        self.call(|client| {
            client
                .put(format!(
                    "{}/{TEAM_GROUP_ID}",
                    self.url_on_main_host("groups")
                ))
                .body(format!(r#"{{"name":"{TEAM_GROUP_NAME}"}}"#))
        })
        .await?;
        Ok(())
    }

    async fn update_team_members(&self, method: Method, jid: &BareJid) -> Result<(), Error> {
        let add_member = |client: &HttpClient| {
            client.request(
                method,
                format!(
                    "{}/{TEAM_GROUP_ID}/members/{}",
                    self.url_on_main_host("groups"),
                    urlencoding::encode(
                        jid.node()
                            .expect("Bare JID has no node part: {jid}")
                            .as_str()
                    )
                ),
            )
        };
        let map_res = |response: Response| async {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or("<nil>".into());
            if status.is_success() {
                Ok(Ok(()))
            } else if body.as_str() == r#"{"result":"group-not-found"}"# {
                Ok(Err(AddMemberFailed::GroupNotFound))
            } else {
                Err(ResponseData {
                    status,
                    headers,
                    body,
                })
            }
        };

        // Try to add the member
        match self.call_(add_member.clone(), map_res).await? {
            Ok(_) => Ok(()),
            // If group wasn't found, try to create it and add the member again
            Err(AddMemberFailed::GroupNotFound) => {
                self.create_team_group().await?;
                self.call(add_member).await?;
                Ok(())
            }
        }
    }
}

struct ResponseData {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

enum AddMemberFailed {
    GroupNotFound,
}

#[async_trait::async_trait]
impl ServerCtlImpl for ProsodyAdminRest {
    async fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error> {
        let mut file = File::create(&self.config_file_path)
            .map_err(|e| Error::CannotOpenConfigFile(self.config_file_path.clone(), e))?;
        let prosody_config = prosody_config_from_db(server_config.to_owned(), app_config);
        file.write_all(prosody_config.to_string().as_bytes())
            .map_err(|e| Error::CannotWriteConfigFile(self.config_file_path.clone(), e))?;

        Ok(())
    }
    async fn reload(&self) -> Result<(), Error> {
        self.call(|client| client.put(self.url("reload")))
            .await
            .map(|_| ())
    }

    async fn add_user(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error> {
        // Create the user
        self.call(|client| {
            client
                .post(format!(
                    "{}/{}",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string())
                ))
                .body(format!(r#"{{"password":"{}"}}"#, password.expose_secret()))
        })
        .await?;

        // Add the user to everyone's roster
        self.update_team_members(Method::PUT, jid).await?;

        Ok(())
    }
    async fn remove_user(&self, jid: &BareJid) -> Result<(), Error> {
        // Remove the user from everyone's roster
        self.update_team_members(Method::DELETE, jid).await?;

        // Delete the user
        self.call(|client| {
            client.delete(format!(
                "{}/{}",
                self.url("user"),
                urlencoding::encode(&jid.to_string())
            ))
        })
        .await?;

        Ok(())
    }
    async fn set_user_role(&self, jid: &BareJid, role: &MemberRole) -> Result<(), Error> {
        self.call(|client| {
            client
                .patch(format!(
                    "{}/{}/role",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string()),
                ))
                .body(format!(r#"{{"role":"{}"}}"#, role.as_prosody()))
        })
        .await
        .map(|_| ())
    }
}

#[async_trait::async_trait]
impl NonStandardXmppClient for ProsodyAdminRest {
    async fn is_connected(&self, jid: &BareJid) -> Result<bool, anyhow::Error> {
        let response = self
            .call(|client| {
                client.get(format!(
                    "{}/{}/connected",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string()),
                ))
            })
            .await?;
        let body = response.text().await?;
        let res: ConnectedResponse = serde_json::from_str(&body)?;
        Ok(res.connected)
    }
}

#[derive(Debug, Deserialize)]
struct ConnectedResponse {
    connected: bool,
}
