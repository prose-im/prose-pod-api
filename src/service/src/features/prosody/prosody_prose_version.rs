// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use tracing::trace;

use crate::{
    errors::{RequestData, ResponseData, UnexpectedHttpResponse},
    pod_version::VersionInfo,
    xmpp::server_ctl,
    AppConfig,
};

/// Rust interface to [`mod_prose_version`](https://github.com/prose-im/prose-pod-server/tree/master/plugins/prose/mod_prose_version.lua).
#[derive(Debug, Clone)]
pub struct ProsodyProseVersion {
    http_client: HttpClient,
    api_url: String,
}

impl ProsodyProseVersion {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            api_url: format!("{}/prose_version", config.server.http_url()),
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
        let request = make_req(&client).build()?;
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
        format!("{}/{path}", self.api_url)
    }

    pub async fn server_version(&self) -> Result<VersionInfo, server_ctl::Error> {
        let response = self.call(|client| client.get(self.url(""))).await?;
        (response.deserialize())
            .map_err(|err| server_ctl::Error::Other(format!("Cannot deserialize: {err}")))
    }
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
        .unwrap_or("Prosody prose_version call failed.".to_string())
}
