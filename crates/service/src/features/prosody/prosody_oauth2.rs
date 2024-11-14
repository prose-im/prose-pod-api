// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use tracing::debug;

use crate::{
    errors::{RequestData, ResponseData, UnexpectedHttpResponse},
    models::BareJid,
    AppConfig,
};

/// Rust interface to [`mod_http_oauth2`](https://hg.prosody.im/prosody-modules/file/tip/mod_http_oauth2).
#[derive(Debug, Clone)]
pub struct ProsodyOAuth2 {
    http_client: HttpClient,
    base_url: String,
}

impl ProsodyOAuth2 {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            base_url: config.server.oauth2_api_url(),
        }
    }

    async fn call(
        &self,
        make_req: impl FnOnce(&HttpClient) -> RequestBuilder,
        accept: impl FnOnce(&ResponseData) -> bool,
    ) -> Result<ResponseData, Error> {
        let client = self.http_client.clone();
        let request = make_req(&client)
            .build()
            .map_err(Error::CannotBuildRequest)?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        let request_data = match request.try_clone() {
            Some(request_clone) => Some(RequestData::from(request_clone).await),
            None => None,
        };
        let response = {
            let response = client.execute(request).await.map_err(Error::CallFailed)?;
            ResponseData::from(response).await
        };

        if accept(&response) {
            Ok(response)
        } else {
            Err(match response.status {
                StatusCode::UNAUTHORIZED => Error::Unauthorized(response.text()),
                StatusCode::FORBIDDEN => Error::Forbidden(response.text()),
                _ => Error::UnexpectedResponse(
                    UnexpectedHttpResponse::new(request_data, response, error_description).await,
                ),
            })
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base_url)
    }
}

impl ProsodyOAuth2 {
    /// Returns an OAuth2 token or `None` if credentials are incorrect.
    /// Returns `Err` if the call failed.
    pub async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<Option<SecretString>, ProsodyOAuth2Error> {
        let jid = jid.to_string();
        let response = self
            .call(
                |client| {
                    client
                        .post(self.url("token"))
                        .basic_auth(jid.clone(), Some(password.expose_secret()))
                        .header("Content-Type", "application/x-www-form-urlencoded")
                        .body(
                            form_urlencoded::Serializer::new(String::new())
                                .append_pair("grant_type", "password")
                                .append_pair("username", jid.clone().as_str())
                                .append_pair("password", password.expose_secret())
                                .append_pair("scope", "xmpp")
                                .finish(),
                        )
                },
                |res| res.status.is_success() || res.status == StatusCode::UNAUTHORIZED,
            )
            .await?;

        if response.status == StatusCode::UNAUTHORIZED {
            debug!("Prosody OAuth2 API returned status {}", response.status);
            return Ok(None);
        }

        let res: ProsodyOAuth2TokenResponse = serde_json::from_str(&response.text())?;

        debug!("Logged in as {jid}");

        Ok(Some(res.access_token))
    }
}

/// Example value:
///
/// ```json
/// {
///   "scope": "",
///   "expires_in": 3600,
///   "token_type": "bearer",
///   "refresh_token": "secret-token:MjswYm5NamVYb3RfcjA7oz+tTt2tLVp1KnY3yBaGWP+MO3JvYmluX3JvYmVydHM5N0B0b3VnaC1vdmVyc2lnaHQub3Jn",
///   "access_token": "secret-token:MjswYm5NamVYb3RfcjA7ErQtXU5WxeQRK6ypyKSTmTizO3JvYmluX3JvYmVydHM5N0B0b3VnaC1vdmVyc2lnaHQub3Jn"
/// }
/// ```
#[derive(Debug, Deserialize)]
struct ProsodyOAuth2TokenResponse {
    // scope: String,
    // expires_in: u16,
    // token_type: String,
    // refresh_token: String,
    access_token: SecretString,
    // grant_jid: String,
}

type Error = ProsodyOAuth2Error;

#[derive(Debug, thiserror::Error)]
pub enum ProsodyOAuth2Error {
    #[error("Cannot build request: {0}")]
    CannotBuildRequest(reqwest::Error),
    #[error("Prosody OAuth2 API call failed: {0}")]
    CallFailed(reqwest::Error),
    #[error("Could not decode Prosody OAuth2 API response: {0}")]
    InvalidResponse(#[from] serde_json::Error),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Unexpected API response: {0}")]
    UnexpectedResponse(UnexpectedHttpResponse),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

fn error_description(json: Option<serde_json::Value>, text: Option<String>) -> String {
    json.clone()
        .map(|json| {
            json.get("error_description")
                .map(|v| v.as_str())
                .flatten()
                .map(ToString::to_string)
        })
        .flatten()
        .or(text.clone())
        .unwrap_or("Prosody http_oauth2 call failed.".to_string())
}
