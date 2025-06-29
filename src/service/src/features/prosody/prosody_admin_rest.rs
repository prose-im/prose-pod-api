// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{sync::Arc, time::Duration};

use mime::Mime;
use reqwest::{Client as HttpClient, Method, RequestBuilder, StatusCode};
use secrecy::ExposeSecret as _;
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, trace};

use crate::{
    errors::{RequestData, ResponseData, UnexpectedHttpResponse},
    secrets::SecretsStore,
    util::DebouncedNotify,
    xmpp::{server_ctl, BareJid, NonStandardXmppClient},
    AppConfig,
};

const TEAM_GROUP_ID: &'static str = "team";
const TEAM_GROUP_NAME: &'static str = "Team";
/// NOTE: Value must be greater than the time it takes to add a member (approx.
///   150ms) otherwise it really is useless but should also be lower than the
///   time it takes for someone to fill a signup form so rosters are updated a
///   bit more frequently.
const TEAM_ROSTERS_SYNC_DEBOUNCE_MILLIS: u64 = 10_000;

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug, Clone)]
pub struct ProsodyAdminRest {
    http_client: HttpClient,
    admin_rest_api_url: String,
    admin_rest_api_on_main_host_url: String,
    api_jid: BareJid,
    secrets_store: SecretsStore,
    team_updated_notifier: Arc<RwLock<Option<(DebouncedNotify, JoinHandle<()>)>>>,
}

impl ProsodyAdminRest {
    pub fn from_config(
        config: &AppConfig,
        http_client: HttpClient,
        secrets_store: SecretsStore,
    ) -> Self {
        Self {
            http_client,
            admin_rest_api_url: config.server.admin_rest_api_url(),
            admin_rest_api_on_main_host_url: config.server.admin_rest_api_on_main_host_url(),
            api_jid: config.api_jid(),
            secrets_store,
            team_updated_notifier: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn call(
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

    pub fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_url)
    }
    pub fn url_on_main_host(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_on_main_host_url)
    }

    pub async fn create_team_group(&self) -> Result<(), server_ctl::Error> {
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

    pub async fn update_team_members(
        &self,
        method: Method,
        jid: &BareJid,
    ) -> Result<(), server_ctl::Error> {
        let add_member = |client: &HttpClient| {
            client
                .request(
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
                .json(&json!({ "delay_update": true }))
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
            Ok(_) => {}
            // If group wasn't found, try to create it and add the member again
            Err(AddMemberFailed::GroupNotFound) => {
                self.create_team_group().await?;
                self.call(add_member).await?;
            }
        }

        // Notify that the team has been updated, to trigger a sync of rosters
        // after a debounce delay.
        self.notify_team_updated().await;

        Ok(())
    }

    pub(crate) async fn update_rosters(&self) -> Result<(), server_ctl::Error> {
        tracing::debug!("Synchronizing rosters…");
        self.call(|client| {
            client.post(format!(
                "{}/{TEAM_GROUP_ID}/sync",
                self.url_on_main_host("groups")
            ))
        })
        .await?;
        Ok(())
    }

    async fn notify_team_updated(&self) {
        {
            if let Some((notifier, _)) = self.team_updated_notifier.read().await.as_ref() {
                notifier.notify();
                return;
            }
        }

        let notifier = DebouncedNotify::new();

        let admin_rest = self.clone();
        let handle = notifier.listen_debounced(
            Duration::from_millis(TEAM_ROSTERS_SYNC_DEBOUNCE_MILLIS),
            move |_| {
                let admin_rest = admin_rest.clone();
                tokio::spawn(async move {
                    if let Err(err) = admin_rest.update_rosters().await {
                        error!("Could not synchronize rosters after updating team members: {err}")
                    }
                });
            },
        );

        notifier.notify();
        *self.team_updated_notifier.write().await = Some((notifier, handle));
    }
}

enum AddMemberFailed {
    GroupNotFound,
}

#[async_trait::async_trait]
impl NonStandardXmppClient for ProsodyAdminRest {
    async fn is_connected(&self, jid: &BareJid) -> Result<bool, anyhow::Error> {
        let response = self
            .call_(
                |client| {
                    client.get(format!(
                        "{}/{}/connected",
                        self.url("user"),
                        urlencoding::encode(&jid.to_string()),
                    ))
                },
                |response| {
                    // Accept the response if it's a 404, as the API returns a 404
                    // when the user has no session (<=> not connected).
                    if response.status.is_success() || response.status == StatusCode::NOT_FOUND {
                        Ok(response)
                    } else {
                        Err(response)
                    }
                },
            )
            .await?;
        let res: ProsodyAdminRestApiResponse<ConnectedResponse> = response.deserialize()?;
        Ok(res.result.connected)
    }
}

#[derive(Debug, Deserialize)]
struct ProsodyAdminRestApiResponse<T> {
    result: T,
}

#[derive(Debug, Deserialize)]
struct ConnectedResponse {
    connected: bool,
}

fn error_description(
    content_type: Option<Mime>,
    json: Option<serde_json::Value>,
    text: Option<String>,
) -> String {
    json.as_ref()
        .map(|v| v.as_str())
        .flatten()
        .map(ToString::to_string)
        .or_else(|| {
            let mime = content_type.unwrap_or(mime::STAR_STAR);
            if mime.essence_str() == "text/html" {
                Some(format!("`{mime}` content"))
            } else {
                text.clone()
            }
        })
        .unwrap_or("Prosody admin_rest call failed.".to_string())
}
