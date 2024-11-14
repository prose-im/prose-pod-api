// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Write as _, path::PathBuf};

use reqwest::{Client as HttpClient, Method, RequestBuilder, StatusCode};
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use serde_json::json;
use tracing::trace;

use super::{prosody_config_from_db, AsProsody as _};
use crate::{
    errors::{RequestData, ResponseData, UnexpectedHttpResponse},
    members::MemberRole,
    secrets::SecretsStore,
    server_config::ServerConfig,
    xmpp::{server_ctl, BareJid, NonStandardXmppClient, ServerCtlImpl},
    AppConfig,
};

const TEAM_GROUP_ID: &'static str = "team";
const TEAM_GROUP_NAME: &'static str = "Team";

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug, Clone)]
pub struct ProsodyAdminRest {
    http_client: HttpClient,
    config_file_path: PathBuf,
    admin_rest_api_url: String,
    admin_rest_api_on_main_host_url: String,
    api_jid: BareJid,
    secrets_store: SecretsStore,
}

impl ProsodyAdminRest {
    pub fn from_config(
        config: &AppConfig,
        http_client: HttpClient,
        secrets_store: SecretsStore,
    ) -> Self {
        Self {
            http_client,
            config_file_path: config.server.prosody_config_file_path.to_owned(),
            admin_rest_api_url: config.server.admin_rest_api_url(),
            admin_rest_api_on_main_host_url: config.server.admin_rest_api_on_main_host_url(),
            api_jid: config.api_jid(),
            secrets_store,
        }
    }

    async fn call(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
    ) -> Result<ResponseData, server_ctl::Error> {
        self.call_(make_req, |response| {
            if response.status.is_success() {
                Ok(response)
            } else {
                Err(response)
            }
        })
        .await
    }

    async fn call_<T>(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        map_res: impl FnOnce(ResponseData) -> Result<T, ResponseData>,
    ) -> Result<T, server_ctl::Error> {
        let client = self.http_client.clone();
        let request = make_req(&client)
            .basic_auth(
                self.api_jid.to_string(),
                Some(
                    self.secrets_store
                        .prose_pod_api_xmpp_password()
                        .expose_secret(),
                ),
            )
            .build()?;
        trace!("Calling `{} {}`…", request.method(), request.url());

        let request_data = match request.try_clone() {
            Some(request_clone) => Some(RequestData::from(request_clone).await),
            None => None,
        };
        let response = {
            let response = client.execute(request).await.map_err(|err| {
                server_ctl::Error::Other(format!("Prosody Admin REST API call failed: {err}"))
            })?;
            ResponseData::from(response).await
        };

        match map_res(response) {
            Ok(res) => Ok(res),
            Err(response) => Err(match response.status {
                StatusCode::UNAUTHORIZED => server_ctl::Error::Unauthorized(response.text()),
                StatusCode::FORBIDDEN => server_ctl::Error::Forbidden(response.text()),
                _ => server_ctl::Error::UnexpectedResponse(
                    UnexpectedHttpResponse::new(request_data, response, error_description).await,
                ),
            }),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_url)
    }
    fn url_on_main_host(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_on_main_host_url)
    }

    async fn create_team_group(&self) -> Result<(), server_ctl::Error> {
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

    async fn update_team_members(
        &self,
        method: Method,
        jid: &BareJid,
    ) -> Result<(), server_ctl::Error> {
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
        let map_res = |response: ResponseData| {
            if response.status.is_success() {
                Ok(Ok(()))
            } else if response
                .body
                .as_ref()
                .is_ok_and(|body| body == &json!({ "result": "group-not-found" }))
            {
                Ok(Err(AddMemberFailed::GroupNotFound))
            } else {
                Err(response)
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

enum AddMemberFailed {
    GroupNotFound,
}

#[async_trait::async_trait]
impl ServerCtlImpl for ProsodyAdminRest {
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
        self.call(|client| client.put(self.url("reload")))
            .await
            .map(|_| ())
    }

    async fn add_user(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<(), server_ctl::Error> {
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

        Ok(())
    }
    async fn remove_user(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
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

    async fn set_user_role(
        &self,
        jid: &BareJid,
        role: &MemberRole,
    ) -> Result<(), server_ctl::Error> {
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
    async fn set_user_password(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<(), server_ctl::Error> {
        self.call(|client| {
            client
                .patch(format!(
                    "{}/{}/password",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string())
                ))
                .body(format!(r#"{{"password":"{}"}}"#, password.expose_secret()))
        })
        .await
        .map(|_| ())
    }

    async fn add_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.update_team_members(Method::PUT, jid).await?;
        Ok(())
    }
    async fn remove_team_member(&self, jid: &BareJid) -> Result<(), server_ctl::Error> {
        self.update_team_members(Method::DELETE, jid).await?;
        Ok(())
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
        let res: ConnectedResponse = response.deserialize()?;
        Ok(res.connected)
    }
}

#[derive(Debug, Deserialize)]
struct ConnectedResponse {
    connected: bool,
}

fn error_description(json: Option<serde_json::Value>, text: Option<String>) -> String {
    json.as_ref()
        .map(|v| v.as_str())
        .flatten()
        .map(ToString::to_string)
        .or(text.clone())
        .unwrap_or("Prosody admin_rest call failed.".to_string())
}
