// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use reqwest::Client as HttpClient;
use tracing::trace;

use crate::{auth::AuthToken, xmpp::jid::NodeRef, AppConfig};

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

    pub async fn add_group_member(
        &self,
        group_id: &str,
        username: &NodeRef,
        token: &AuthToken,
    ) -> Result<(), anyhow::Error> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http.put(url).bearer_auth(token.expose_secret()).build()?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = http.execute(request).await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.ok();
            return Err(anyhow::Error::msg(format!(
                "admin_api call failed: {status}: {body:?}"
            )));
        }

        Ok(())
    }

    pub async fn remove_group_member(
        &self,
        group_id: &str,
        username: &NodeRef,
        token: &AuthToken,
    ) -> Result<(), anyhow::Error> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http
            .delete(url)
            .bearer_auth(token.expose_secret())
            .build()?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = http.execute(request).await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.ok();
            return Err(anyhow::Error::msg(format!(
                "admin_api call failed: {status}: {body:?}"
            )));
        }

        Ok(())
    }
}
