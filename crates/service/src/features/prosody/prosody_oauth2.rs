// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use reqwest::{Client as HttpClient, RequestBuilder, Response, StatusCode};
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use tracing::debug;

use crate::{model::BareJid, AppConfig};

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
        accept: impl FnOnce(&Response) -> bool,
    ) -> Result<Response, Error> {
        let client = self.http_client.clone();
        let request = make_req(&client)
            .build()
            .map_err(Error::CannotBuildRequest)?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        let (response, request_clone) = {
            let request_clone = request.try_clone();
            (
                client.execute(request).await.map_err(Error::CallFailed)?,
                request_clone,
            )
        };
        if accept(&response) {
            Ok(response)
        } else {
            let mut err = format!(
                "Prosody OAuth2 API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                response.status(),
                response.headers().clone(),
                response.text().await.unwrap_or("<nil>".to_string()),
            );
            if let Some(request) = request_clone {
                err.push_str(&format!(
                    "\n  Request headers: {:?}\n  Request body: {:?}",
                    request.headers().clone(),
                    request
                        .body()
                        .and_then(|body| body.as_bytes())
                        .map(std::str::from_utf8),
                ));
            }
            Err(Error::UnexpectedResponse(err))
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
                |res| res.status().is_success() || res.status() == StatusCode::UNAUTHORIZED,
            )
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            debug!("Prosody OAuth2 API returned status {}", response.status());
            return Ok(None);
        }

        let body = response
            .text()
            .await
            .map_err(|e| Error::UnexpectedResponse(format!("Could not read response body: {e}")))?;

        let res: ProsodyOAuth2TokenResponse = serde_json::from_str(&body)?;

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
    #[error("Unexpected Prosody OAuth2 API response: {0}")]
    UnexpectedResponse(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}
