// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;
use serde_json::json;
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

    pub async fn call(
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
                StatusCode::BAD_REQUEST
                    if response.text().to_lowercase().contains("invalid jid") =>
                {
                    Error::Unauthorized("Invalid JID".to_string())
                }
                StatusCode::BAD_REQUEST
                    if response
                        .text()
                        .to_lowercase()
                        .contains("incorrect credentials") =>
                {
                    Error::Unauthorized("Incorrect credentials".to_string())
                }
                StatusCode::BAD_REQUEST
                    if response.text().to_lowercase().contains("invalid_grant") =>
                {
                    Error::Unauthorized("Invalid token".to_string())
                }
                _ => Error::UnexpectedResponse(
                    UnexpectedHttpResponse::new(request_data, response, error_description).await,
                ),
            })
        }
    }

    pub fn url(&self, path: &str) -> String {
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
                                // DOC: Space-separated list of scopes the client promises to restrict itself to.
                                //   Supported scopes: [
                                //     "prosody:operator", "prosody:admin", "prosody:member", "prosody:registered", "prosody:guest",
                                //     "xmpp", "openid"
                                //   ]
                                // NOTE: "openid" scope required for `/userinfo` route.
                                .append_pair("scope", "xmpp openid")
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

        let res: TokenResponse = serde_json::from_str(&response.text())?;

        debug!("Logged in as {jid}");

        Ok(Some(res.access_token))
    }

    pub async fn register(&self) -> Result<(), Error> {
        let response = self
            .call(
                |client| {
                    client.post(self.url("register")).json(&json!({
                        "client_name": "Prose Pod API",
                        "client_uri": "https://prose-pod-api:8080",
                        "redirect_uris": [
                            "https://prose-pod-api:8080/redirect"
                        ],
                    }))
                },
                |res| res.status.is_success(),
            )
            .await?;

        let _: RegisterResponse = serde_json::from_str(&response.text())?;

        Ok(())
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
struct TokenResponse {
    // scope: String,
    // expires_in: u16,
    // token_type: String,
    // refresh_token: String,
    access_token: SecretString,
    // grant_jid: String,
}

/// Example value:
///
/// ```json
/// {
///     "application_type": "web",
///     "client_secret": "3e841d9de79645a1c4c3f82f5d59485531b7e03119d8622acc31dc243baff2a5",
///     "redirect_uris": [
///         "https://prose-pod-api:8080/redirect"
///     ],
///     "iat": 1731265591,
///     "nonce": "Rxd4I7sqjsFq",
///     "response_types": [
///         "code"
///     ],
///     "exp": 1731269191,
///     "client_uri": "https://prose-pod-api:8080",
///     "client_name": "Prose Pod API",
///     "client_id_issued_at": 1731265591,
///     "client_id":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhcHBsaWNhdGlvbl90eXBlIjoid2ViIiwiaWF0IjoxNzMxMjY1NTkxLCJjbGllbnRfbmFtZSI6IlByb3NlIFBvZCBBUEkiLCJyZXwb25zZV90eXBlcyI6WyJjb2RlIl0sImNsaWVudF91cmkiOiJodHRwczovL3Byb3NlLXBvZC1hcGk6ODA4MCIsImdyYW50X3R5cGVzIjpbImF1dGhvcml6YXRpb25fY29kZSJdLCJyZWRcmVjdF91cmlzIjpbImh0dHBzOi8vcHJvc2UtcG9kLWFwaTo4MDgwL3JlZGlyZWN0Il0sIm5vbmNlIjoiUnhkNEk3c3Fqc0ZxIiwidG9rZW5fZW5kcG9pbnRfYXV0aF9tZXRob2QiOiJjGllbnRfc2VjcmV0X2Jhc2ljIiwiZXhwIjoxNzMxMjY5MTkxfQ.-4b6hnqllAzH9TjzaRhQWbJ09cGuVs-8hXB05yLx1Qo",
///     "grant_types": [
///         "authorization_code"
///     ],
///     "token_endpoint_auth_method": "client_secret_basic",
///     "client_secret_expires_at": 0
/// }
/// ```
#[derive(Debug, Deserialize)]
struct RegisterResponse {}

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

fn error_description(
    content_type: Option<Mime>,
    json: Option<serde_json::Value>,
    text: Option<String>,
) -> String {
    json.clone()
        .map(|json| {
            json.get("error_description")
                .map(|v| v.as_str())
                .flatten()
                .map(ToString::to_string)
        })
        .flatten()
        .or_else(|| {
            let mime = content_type.unwrap_or(mime::STAR_STAR);
            if mime.essence_str() == "text/html" {
                Some(format!("`{mime}` content"))
            } else {
                text.clone()
            }
        })
        .unwrap_or("Prosody http_oauth2 call failed.".to_string())
}
