// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{sync::Arc, time::Duration};

use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use secrecy::ExposeSecret as _;
use serdev::Deserialize;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, trace, Instrument as _};

use crate::{
    errors::{RequestData, ResponseData, UnexpectedHttpResponse},
    secrets::SecretsStore,
    util::DebouncedNotify,
    xmpp::{server_ctl, BareJid, NonStandardXmppClient},
    AppConfig,
};

// TODO: Move somewhere else.
pub(crate) const TEAM_GROUP_ID: &'static str = "team";
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
            admin_rest_api_url: format!("{}/admin_rest", config.server.admin_http_url()),
            admin_rest_api_on_main_host_url: format!("{}/admin_rest", config.server.http_url()),
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
                        .expect("Pod API XMPP password not initialized")
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
                server_ctl::Error::Internal(
                    anyhow::Error::new(err).context("Prosody Admin REST API call failed"),
                )
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

    pub async fn list_users(&self) -> Result<Vec<ListUsersItem>, server_ctl::Error> {
        let response = self
            .call(|client| client.get(self.url_on_main_host("all-users")))
            .await?;
        let res: ProsodyAdminRestApiResponse<ListUsersResponse> = (response.deserialize())
            .map_err(|err| {
                server_ctl::Error::Internal(anyhow::Error::new(err).context("Cannot deserialize"))
            })?;
        Ok(res.result.users)
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
                tokio::spawn(
                    async move {
                        if let Err(err) = admin_rest.update_rosters().await {
                            error!(
                                "Could not synchronize rosters after updating team members: {err}"
                            )
                        }
                    }
                    .in_current_span(),
                );
            },
        );

        notifier.notify();
        *self.team_updated_notifier.write().await = Some((notifier, handle));
    }
}

#[derive(Debug)]
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

#[derive(Debug, Deserialize)]
struct ListUsersResponse {
    // count: u32,
    users: Vec<ListUsersItem>,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersItem {
    pub jid: BareJid,
    pub role: Role,
    // pub secondary_roles: Vec<Role>,
}

#[derive(Debug, Deserialize)]
pub struct Role {
    /// E.g. `"EHKt_OKcF-5K"`.
    pub id: String,
    /// E.g. `"prosody:member"`.
    pub name: String,
    /// E.g. `35`.
    pub priority: i16,
    #[serde(default)]
    pub inherits: Vec<Role>,
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
