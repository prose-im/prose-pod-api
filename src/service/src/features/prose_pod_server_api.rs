// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use bytes::Bytes;
use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, Response, StatusCode};
use secrecy::SecretString;
use serde_json::json;
use serdev::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::trace;

use crate::{
    auth::{AuthToken, UserInfo},
    errors::{Forbidden, RequestData, ResponseData, Unauthorized, UnexpectedHttpResponse},
    init::errors::FirstAccountAlreadyCreated,
    models::{Avatar, Color},
    prosody::ProsodyRoleName,
    util::either::Either,
    workspace::errors::WorkspaceAlreadyInitialized,
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
    #[error("{0:#}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct ProsePodServerApiError {
    pub kind: String,
    pub code: String,
    pub message: String,
    pub description: String,
}

impl ProsePodServerApi {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            api_url: config.server_api_url(),
        }
    }
}

impl ProsePodServerApi {
    pub async fn health(&self) -> Result<(), ProsePodServerError> {
        let response = self
            .call(|client| client.get(self.url("/health")), None)
            .await?;
        assert!(response.status().is_success());
        Ok(())
    }

    pub async fn is_healthy(&self) -> Result<bool, ProsePodServerError> {
        match self.health().await {
            Ok(()) => Ok(true),
            Err(ProsePodServerError::Unavailable) => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub async fn lifecycle_reload(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<(), ProsePodServerError> {
        let response = self
            .call(|client| client.post(self.url("/lifecycle/reload")), auth)
            .await?;
        assert!(response.status().is_success());
        Ok(())
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
                    client.put(self.url("/init/first-account")).json(&json!({
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
        self.get("/users-util/stats", auth).await
    }

    pub async fn users_util_admin_jids(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Vec<BareJid>, ProsePodServerError> {
        self.get("/users-util/admin-jids", auth).await
    }

    pub async fn users_util_self(&self, auth: &AuthToken) -> Result<UserInfo, ProsePodServerError> {
        self.get("/users-util/self", Some(auth)).await
    }

    pub async fn invitations_util_stats(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<GetInvitationsStatsResponse, ProsePodServerError> {
        self.get("/invitations-util/stats", auth).await
    }
}

impl ProsePodServerApi {
    pub async fn init_workspace(
        &self,
        req: &InitWorkspaceRequest,
    ) -> Result<(), Either<WorkspaceAlreadyInitialized, ProsePodServerError>> {
        let response = self
            .call_(
                |client| client.put(self.url("/workspace-init")).json(req),
                |response| {
                    let status = response.status();
                    if status.is_success() || status == StatusCode::GONE {
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
            status if status.is_success() => Ok(()),
            StatusCode::GONE => Err(Either::E1(WorkspaceAlreadyInitialized)),
            _ => unreachable!(),
        }
    }

    pub async fn get_workspace(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<GetWorkspaceResponse, ProsePodServerError> {
        self.get("/workspace", auth).await
    }

    pub async fn patch_workspace(
        &self,
        req: &PatchWorkspaceRequest,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.call(
            |client| client.patch(self.url("/workspace")).json(req),
            Some(auth),
        )
        .await?;
        Ok(())
    }

    pub async fn get_workspace_name(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<String, ProsePodServerError> {
        self.get("/workspace/name", auth).await
    }

    pub async fn set_workspace_name(
        &self,
        name: &str,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.put("/workspace/name", name, auth).await
    }

    pub async fn get_workspace_accent_color(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Option<Color>, ProsePodServerError> {
        self.get("/workspace/accent-color", auth).await
    }

    pub async fn set_workspace_accent_color(
        &self,
        accent_color: &Option<Color>,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.put("/workspace/accent-color", accent_color, auth)
            .await
    }

    pub async fn get_workspace_icon(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Option<Avatar>, ProsePodServerError> {
        self.get("/workspace/icon", auth).await
    }

    pub async fn set_workspace_icon(
        &self,
        icon: Avatar,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.put_raw("/workspace/icon", icon.into_inner(), auth)
            .await
    }
}

#[derive(Serialize)]
pub struct InitWorkspaceRequest {
    pub name: String,
    pub accent_color: Option<Color>,
}

#[derive(Deserialize)]
pub struct GetWorkspaceResponse {
    name: String,
    icon: Option<Avatar>,
    accent_color: Option<Color>,
}

#[derive(Default)]
#[derive(Serialize)]
pub struct PatchWorkspaceRequest {
    pub name: Option<String>,
    pub accent_color: Option<Option<Color>>,
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
    #[inline]
    async fn get<T: DeserializeOwned>(
        &self,
        path: &'static str,
        auth: Option<&AuthToken>,
    ) -> Result<T, ProsePodServerError> {
        let response = self.call(|client| client.get(self.url(path)), auth).await?;

        let response: T = response.json().await.context("Invalid response body")?;

        Ok(response)
    }

    #[inline]
    async fn put<T: Serialize + ?Sized>(
        &self,
        path: &'static str,
        body: &T,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.call(|client| client.put(self.url(path)).json(body), Some(auth))
            .await?;
        Ok(())
    }

    #[inline]
    async fn put_raw(
        &self,
        path: &'static str,
        body: Bytes,
        auth: &AuthToken,
    ) -> Result<(), ProsePodServerError> {
        self.call(|client| client.put(self.url(path)).body(body), Some(auth))
            .await?;
        Ok(())
    }

    pub fn url(&self, path: &str) -> String {
        assert!(path.starts_with("/"), "path={path:?}");
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

// MARK: - Domain conversions

impl From<GetWorkspaceResponse> for super::workspace::Workspace {
    fn from(res: GetWorkspaceResponse) -> Self {
        Self {
            name: res.name,
            icon: res.icon,
            accent_color: res.accent_color,
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
            if mime.essence_str().starts_with("text/html") {
                Some(format!("`{mime}` content"))
            } else {
                text.clone()
            }
        })
        .unwrap_or("Prose Pod Server API call failed.".to_string())
}
