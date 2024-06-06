// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use log::debug;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use tokio::runtime::Handle;

use crate::config::Config;

/// Rust interface to [`mod_http_oauth2`](https://hg.prosody.im/prosody-modules/file/tip/mod_http_oauth2).
#[derive(Debug, Clone)]
pub struct ProsodyOAuth2 {
    base_url: String,
}

impl ProsodyOAuth2 {
    pub fn from_config(config: &Config) -> Self {
        Self {
            base_url: config.server.oauth2_api_url(),
        }
    }

    fn call(
        &self,
        make_req: impl FnOnce(&Client) -> RequestBuilder,
        accept: impl FnOnce(&Response) -> bool,
    ) -> Result<Response, Error> {
        let client = Client::new();
        let request = make_req(&client)
            .build()
            .map_err(Error::CannotBuildRequest)?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let response = client.execute(request).await
                    .map_err(Error::CallFailed)?;
                if accept(&response) {
                    Ok(response)
                } else {
                    Err(Error::UnexpectedResponse(format!(
                        "Prosody OAuth2 API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                        response.status(),
                        response.headers().clone(),
                        response.text().await.unwrap_or("<nil>".to_string())
                    )))
                }
            })
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base_url)
    }
}

impl ProsodyOAuth2 {
    /// Returns an OAuth2 token or `None` if credentials are incorrect.
    /// Returns `Err` if the call failed.
    pub fn log_in(&self, jid: &JID, password: &str) -> Result<Option<String>, ProsodyOAuth2Error> {
        let response = self.call(
            |client| {
                client
                    .get(self.url("token"))
                    .basic_auth(jid, Some(password))
            },
            |res| res.status().is_success() || res.status() == StatusCode::UNAUTHORIZED,
        )?;

        if response.status() == StatusCode::UNAUTHORIZED {
            debug!("Prosody OAuth2 API returned status {}", response.status());
            return Ok(None);
        }

        let body = tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                response.text().await.map_err(|e| {
                    Error::UnexpectedResponse(format!("Could not read response body: {e}"))
                })
            })
        })?;

        dbg!(body);
        todo!()
    }
}

type Error = ProsodyOAuth2Error;

#[derive(Debug, thiserror::Error)]
pub enum ProsodyOAuth2Error {
    #[error("Cannot build request: {0}")]
    CannotBuildRequest(reqwest::Error),
    #[error("Prosody OAuth2 API call failed: {0}")]
    CallFailed(reqwest::Error),
    #[error("Unexpected OAuth2 API response: {0}")]
    UnexpectedResponse(String),
}
