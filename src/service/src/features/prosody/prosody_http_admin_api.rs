// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use reqwest::{Client as HttpClient, StatusCode};
use tracing::trace;

use crate::{
    auth::AuthToken,
    errors::Forbidden,
    util::either::{Either, Either3},
    xmpp::{
        jid::NodeRef,
        server_ctl::errors::{GroupAlreadyExists, GroupNotFound},
    },
    AppConfig,
};

#[derive(Debug)]
pub struct ProsodyHttpAdminApi {
    http_client: HttpClient,
    api_root: String,
}

impl ProsodyHttpAdminApi {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            api_root: format!("{}/admin_api", config.server.http_url()),
        }
    }

    pub async fn create_group(
        &self,
        group_id: &str,
        group_name: &str,
        token: &AuthToken,
    ) -> Result<(), Either3<Forbidden, GroupAlreadyExists, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!("{root}/groups", root = self.api_root,);
        let request = http
            .put(url)
            .bearer_auth(token.expose_secret())
            .body(format!(r#"{{"name":"{group_name}","id":"{group_id}"}}"#))
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = (response.text().await)
                .ok()
                .unwrap_or_else(|| "<empty response body>".to_owned());
            match status {
                StatusCode::FORBIDDEN => Err(Either3::E1(Forbidden(body))),
                StatusCode::CONFLICT => Err(Either3::E2(GroupAlreadyExists {
                    group_id: group_id.to_owned(),
                })),
                _ => Err(Either3::E3(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}"
                )))),
            }
        }
    }

    pub async fn add_group_member(
        &self,
        group_id: &str,
        username: &NodeRef,
        token: &AuthToken,
    ) -> Result<(), Either3<Forbidden, GroupNotFound, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http
            .put(url)
            .bearer_auth(token.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = (response.text().await)
                .ok()
                .unwrap_or_else(|| "<empty response body>".to_owned());
            match status {
                StatusCode::FORBIDDEN => Err(Either3::E1(Forbidden(body))),
                StatusCode::NOT_FOUND => Err(Either3::E2(GroupNotFound {
                    group_id: group_id.to_owned(),
                })),
                _ => Err(Either3::E3(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}"
                )))),
            }
        }
    }

    pub async fn remove_group_member(
        &self,
        group_id: &str,
        username: &NodeRef,
        token: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http
            .delete(url)
            .bearer_auth(token.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = (response.text().await)
                .ok()
                .unwrap_or_else(|| "<empty response body>".to_owned());
            match status {
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(body))),
                StatusCode::NOT_FOUND => Ok(()),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}"
                )))),
            }
        }
    }
}
