// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use serdev::Deserialize;
use tracing::trace;

use crate::{
    auth::AuthToken,
    errors::{Forbidden, RequestData, ResponseData, UnexpectedHttpResponse},
    members::{Member, MemberRole},
    prosody::ProsodyRole,
    util::either::Either,
    xmpp::{BareJid, NonStandardXmppClient},
    AppConfig, TEAM_GROUP_ID,
};

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug, Clone)]
pub struct ProsodyAdminRest {
    http_client: HttpClient,
    admin_rest_api_url: String,
}

impl ProsodyAdminRest {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            admin_rest_api_url: format!("{}/admin_rest", config.server.http_url()),
        }
    }

    pub async fn call(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        auth: &AuthToken,
    ) -> Result<ResponseData, Either<Forbidden, anyhow::Error>> {
        self.call_(
            make_req,
            |response| {
                if response.status.is_success() {
                    Ok(response)
                } else {
                    Err(response)
                }
            },
            auth,
        )
        .await
    }

    async fn call_<T>(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        map_res: impl FnOnce(ResponseData) -> Result<T, ResponseData>,
        auth: &AuthToken,
    ) -> Result<T, Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let client = self.http_client.clone();
        let request = make_req(&client)
            .bearer_auth(auth.expose_secret())
            .build()
            .context("Error building request")?;
        trace!("Calling `{} {}`…", request.method(), request.url());

        let request_data = match request.try_clone() {
            Some(request_clone) => Some(RequestData::from(request_clone).await),
            None => None,
        };
        let response = {
            let response = client
                .execute(request)
                .await
                .context("Prosody Admin REST API call failed")?;
            ResponseData::from(response).await
        };

        match map_res(response) {
            Ok(res) => Ok(res),
            Err(response) => Err(match response.status {
                StatusCode::FORBIDDEN => Either::E1(Forbidden(response.text())),
                // NOTE: `401 Unauthorized`s can technically happen, but it’d
                //   mean something is not configured properly internally.
                _ => Either::E2(anyhow::Error::new(UnexpectedHttpResponse::new(
                    request_data,
                    response,
                    error_description,
                ))),
            }),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.admin_rest_api_url)
    }

    #[must_use]
    pub async fn list_users(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<ListUsersItem>, Either<Forbidden, anyhow::Error>> {
        let response = self
            .call(|client| client.get(self.url("all-users")), auth)
            .await?;
        let res: ProsodyAdminRestApiResponse<ListUsersResponse> =
            response.deserialize().context("Cannot deserialize")?;
        Ok(res.result.users)
    }

    #[allow(unused)]
    pub(crate) async fn update_rosters(
        &self,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        tracing::debug!("Synchronizing rosters…");
        self.call(
            |client| client.post(format!("{}/{TEAM_GROUP_ID}/sync", self.url("groups"))),
            auth,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl NonStandardXmppClient for ProsodyAdminRest {
    async fn is_connected(&self, jid: &BareJid, auth: &AuthToken) -> Result<bool, anyhow::Error> {
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
                auth,
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
    pub role: Option<ProsodyRole>,
    // pub secondary_roles: Vec<ProsodyRole>,
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

// MARK: - Boilerplate

impl From<ListUsersItem> for Member {
    fn from(info: ListUsersItem) -> Self {
        let role = info.role.expect("Members should have roles");

        Self {
            jid: info.jid,
            role: MemberRole::try_from(&role).expect("Members should have supported roles"),
        }
    }
}
