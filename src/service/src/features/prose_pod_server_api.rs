// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, Response, StatusCode};
use secrecy::SecretString;
use serde_json::json;
use serdev::Deserialize;
use tracing::trace;

use crate::{
    auth::{AuthToken, UserInfo},
    errors::{Forbidden, RequestData, ResponseData, Unauthorized, UnexpectedHttpResponse},
    init::errors::FirstAccountAlreadyCreated,
    prosody::ProsodyRoleName,
    util::either::Either,
    xmpp::jid::{BareJid, JidNode, NodeRef},
    AppConfig,
};

#[derive(Debug, Clone)]
pub struct ProsePodServerApi {
    http_client: HttpClient,
    api_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProsePodServerError {
    #[error("Server unavailable.")]
    Unavailable,
    #[error("{0}")]
    Forbidden(Forbidden),
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl ProsePodServerApi {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            api_url: config.server.api_url(),
        }
    }
}

impl ProsePodServerApi {
    pub async fn health(&self) -> Result<bool, ProsePodServerError> {
        let response = self
            .call_(
                |client| client.get(self.url("/health")),
                |response| {
                    let status = response.status();
                    if status.is_success() || status == StatusCode::SERVICE_UNAVAILABLE {
                        Ok(response)
                    } else {
                        Err(response)
                    }
                },
                None,
            )
            .await?;
        Ok(response.status().is_success())
    }

    pub async fn lifecycle_reload(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<bool, ProsePodServerError> {
        let response = self
            .call(|client| client.put(self.url("/lifecycle/reload")), auth)
            .await?;
        Ok(response.status().is_success())
    }

    pub async fn init_first_account(
        &self,
        username: &NodeRef,
        password: &SecretString,
    ) -> Result<CreateAccountResponse, Either<FirstAccountAlreadyCreated, ProsePodServerError>>
    {
        use secrecy::ExposeSecret as _;

        let response = self
            .call_(
                |client| {
                    client.put("/init/first-account").json(&json!({
                        "username": username.as_str(),
                        "password": password.expose_secret(),
                    }))
                },
                |response| {
                    let status = response.status();
                    if status.is_success() || status == StatusCode::CONFLICT {
                        Ok(response)
                    } else {
                        Err(response)
                    }
                },
                None,
            )
            .await
            .map_err(Either::E2)?;

        match response.status() {
            status if status.is_success() => {
                let response: CreateAccountResponse = response
                    .json()
                    .await
                    .context("Invalid response body")
                    .map_err(ProsePodServerError::Internal)
                    .map_err(Either::E2)?;
                Ok(response)
            }
            StatusCode::CONFLICT => Err(Either::E1(FirstAccountAlreadyCreated)),
            _ => unreachable!(),
        }
    }

    pub async fn users_util_stats(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<GetUsersStatsResponse, ProsePodServerError> {
        let response = self
            .call(|client| client.put(self.url("/users-util/stats")), auth)
            .await?;

        let response: GetUsersStatsResponse =
            response.json().await.context("Invalid response body")?;

        Ok(response)
    }

    pub async fn users_util_admin_jids(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Vec<BareJid>, ProsePodServerError> {
        let response = self
            .call(
                |client| client.put(self.url("/users-util/admin-jids")),
                auth,
            )
            .await?;

        let response: Vec<BareJid> = response.json().await.context("Invalid response body")?;

        Ok(response)
    }

    pub async fn users_util_self(&self, auth: &AuthToken) -> Result<UserInfo, ProsePodServerError> {
        let response = self
            .call(
                |client| client.put(self.url("/users-util/self")),
                Some(auth),
            )
            .await?;

        let response: UserInfo = response.json().await.context("Invalid response body")?;

        Ok(response)
    }

    pub async fn invitations_util_stats(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<GetInvitationsStatsResponse, ProsePodServerError> {
        let response = self
            .call(
                |client| client.put(self.url("/invitations-util/stats")),
                auth,
            )
            .await?;

        let response: GetInvitationsStatsResponse =
            response.json().await.context("Invalid response body")?;

        Ok(response)
    }
}

#[derive(Deserialize)]
pub struct GetUsersStatsResponse {
    pub count: usize,
}

#[derive(Deserialize)]
pub struct GetInvitationsStatsResponse {
    pub count: usize,
}

#[derive(Deserialize)]
pub struct CreateAccountResponse {
    pub username: JidNode,
    pub role: ProsodyRoleName,
}

// MARK: - Helpers

impl ProsePodServerApi {
    pub fn url(&self, path: &str) -> String {
        assert!(!path.starts_with("/"), "path={path:?}");
        format!("{}{path}", self.api_url)
    }

    pub async fn call(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        auth: Option<&AuthToken>,
    ) -> Result<Response, ProsePodServerError> {
        self.call_(
            make_req,
            |response| {
                if response.status().is_success() {
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
        map_res: impl FnOnce(Response) -> Result<T, Response>,
        auth: Option<&AuthToken>,
    ) -> Result<T, ProsePodServerError> {
        use secrecy::ExposeSecret as _;

        let client = self.http_client.clone();
        let mut request_builder = make_req(&client);
        if let Some(auth) = auth {
            request_builder = request_builder.bearer_auth(auth.expose_secret())
        }
        let request = request_builder.build().map_err(anyhow::Error::new)?;
        trace!("Calling `{} {}`…", request.method(), request.url());

        let request_data = match request.try_clone() {
            Some(request_clone) => Some(RequestData::from(request_clone).await),
            None => None,
        };
        let response = {
            let response = client.execute(request).await.map_err(|err| {
                anyhow::Error::new(err).context("Prose Pod Server API call failed")
            })?;
            response
        };

        match map_res(response) {
            Ok(res) => Ok(res),
            Err(response) => {
                let status = response.status();
                let read_response = async || ResponseData::from(response).await;
                Err(match status {
                    StatusCode::SERVICE_UNAVAILABLE => ProsePodServerError::Unavailable,
                    StatusCode::UNAUTHORIZED => ProsePodServerError::Internal(anyhow::Error::new(
                        Unauthorized(read_response().await.text()),
                    )),
                    StatusCode::FORBIDDEN => {
                        ProsePodServerError::Forbidden(Forbidden(read_response().await.text()))
                    }
                    _ => ProsePodServerError::Internal(anyhow::Error::new(
                        UnexpectedHttpResponse::new(
                            request_data,
                            read_response().await,
                            error_description,
                        ),
                    )),
                })
            }
        }
    }
}

// MARK: - Boilerplate

impl<E> From<Result<ResponseData, E>> for ProsePodServerError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(result: Result<ResponseData, E>) -> Self {
        match result {
            Ok(response) if response.status == StatusCode::SERVICE_UNAVAILABLE => Self::Unavailable,
            Ok(response) => Self::Internal(anyhow::Error::new(UnexpectedHttpResponse::new(
                None,
                response,
                error_description,
            ))),
            Err(err) => Self::Internal(anyhow::Error::new(err)),
        }
    }
}

fn error_description(
    content_type: Option<Mime>,
    json: Option<serde_json::Value>,
    text: Option<String>,
) -> String {
    json.as_ref()
        .map(serde_json::Value::as_str)
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
        .unwrap_or("Prose Pod Server API call failed.".to_string())
}
