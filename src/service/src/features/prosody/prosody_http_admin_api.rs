// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use reqwest::{Client as HttpClient, StatusCode};
use serdev::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    auth::AuthToken,
    errors::{Forbidden, GroupAlreadyExists, GroupNotFound},
    invitations::{errors::InvitationNotFound, InvitationId, InvitationToken},
    members::{Member, MemberRole},
    prosody::ProsodyRoleName,
    util::either::{Either, Either3},
    xmpp::{
        jid::{BareJid, NodeRef},
        JidNode,
    },
    AppConfig,
};

/// Rust interface to [`mod_http_admin_api`](https://github.com/prose-im/prose-pod-server/tree/master/plugins/community/mod_http_admin_api).
#[derive(Debug, Clone)]
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
}

// MARK: Users

impl ProsodyHttpAdminApi {
    pub async fn get_user_by_name(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<Option<Member>, Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/users/{username}",
            root = self.api_root,
            username = username.to_string()
        );
        let request = http
            .get(url)
            .bearer_auth(auth.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let user_info: UserInfo = response.json().await.context("Invalid response body")?;
            let member = Member::from(user_info);
            Ok(Some(member))
        } else {
            let body = (response.text().await)
                .ok()
                .unwrap_or_else(|| "<empty response body>".to_owned());
            match status {
                StatusCode::NOT_FOUND => Ok(None),
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(body))),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}"
                )))),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct UserInfo {
    jid: BareJid,
    // username: JidNode,
    // display_name: String,
    role: Option<ProsodyRoleName>,
    // secondary_roles: Vec<ProsodyRoleName>,
    // NOTE: Not yet implemented.
    // email: EmailAddress,
    // NOTE: Not yet implemented.
    // phone: String,
    // groups: Vec<String>,
}

// MARK: Groups

impl ProsodyHttpAdminApi {
    pub async fn create_group(
        &self,
        group_id: &str,
        group_name: &str,
        auth: &AuthToken,
    ) -> Result<(), Either3<Forbidden, GroupAlreadyExists, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!("{root}/groups", root = self.api_root);
        let request = http
            .put(url)
            .bearer_auth(auth.expose_secret())
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
        auth: &AuthToken,
    ) -> Result<(), Either3<Forbidden, GroupNotFound, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http
            .put(url)
            .bearer_auth(auth.expose_secret())
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
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/groups/{group_id}/members/{username}",
            root = self.api_root,
        );
        let request = http
            .delete(url)
            .bearer_auth(auth.expose_secret())
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

// MARK: Invites

impl ProsodyHttpAdminApi {
    pub async fn list_invites(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<InviteInfo>, Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!("{root}/invites", root = self.api_root);
        let request = http
            .get(url)
            .bearer_auth(auth.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let invite_info: Vec<InviteInfo> =
                response.json().await.context("Invalid response body")?;
            Ok(invite_info)
        } else {
            let read_body = async {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(read_body.await))),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}",
                    body = read_body.await
                )))),
            }
        }
    }

    pub async fn create_invite_for_account(
        &self,
        req: CreateAccountInvitationRequest,
        auth: &AuthToken,
    ) -> Result<InviteInfo, Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!("{root}/invites/account", root = self.api_root);
        let request = http
            .post(url)
            .bearer_auth(auth.expose_secret())
            .json(&req)
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let invite_info: InviteInfo = response.json().await.context("Invalid response body")?;
            Ok(invite_info)
        } else {
            let read_body = async {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(read_body.await))),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}",
                    body = read_body.await
                )))),
            }
        }
    }

    pub async fn create_invite_for_account_reset(
        &self,
        req: CreateAccountResetInvitationRequest,
        auth: &AuthToken,
    ) -> Result<InviteInfo, Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!("{root}/invites/reset", root = self.api_root);
        let request = http
            .post(url)
            .bearer_auth(auth.expose_secret())
            .json(&req)
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let invite_info: InviteInfo = response.json().await.context("Invalid response body")?;
            Ok(invite_info)
        } else {
            let read_body = async {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(read_body.await))),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}",
                    body = read_body.await
                )))),
            }
        }
    }

    // NOTE: What `mod_http_admin_api` calls “Invite IDs”
    //   really are invite tokens.
    pub async fn get_invite_by_id(
        &self,
        token: &InvitationToken,
        auth: &AuthToken,
    ) -> Result<InviteInfo, Either3<Forbidden, InvitationNotFound, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/invites/{id}",
            root = self.api_root,
            id = token.expose_secret()
        );
        let request = http
            .get(url)
            .bearer_auth(auth.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let invite_info: InviteInfo = response.json().await.context("Invalid response body")?;
            Ok(invite_info)
        } else {
            let read_body = async {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::FORBIDDEN => Err(Either3::E1(Forbidden(read_body.await))),
                StatusCode::NOT_FOUND => Err(Either3::E2(InvitationNotFound(token.clone()))),
                _ => Err(Either3::E3(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}",
                    body = read_body.await
                )))),
            }
        }
    }

    // NOTE: What `mod_http_admin_api` calls “Invite IDs”
    //   really are invite tokens.
    pub async fn delete_invite(
        &self,
        token: &InvitationToken,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let url = format!(
            "{root}/invites/{id}",
            root = self.api_root,
            id = token.expose_secret()
        );
        let request = http
            .delete(url)
            .bearer_auth(auth.expose_secret())
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let read_body = async {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::FORBIDDEN => Err(Either::E1(Forbidden(read_body.await))),
                StatusCode::NOT_FOUND => Ok(()),
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "admin_api call failed: {status}: {body}",
                    body = read_body.await
                )))),
            }
        }
    }
}

#[derive(Serialize)]
pub struct CreateAccountInvitationRequest {
    pub username: Option<JidNode>,
    #[serde(rename = "ttl")]
    pub ttl_secs: Option<u32>,
    pub groups: Option<Vec<String>>,
    pub roles: Option<Vec<ProsodyRoleName>>,
    pub note: Option<String>,
    pub additional_data: serde_json::Value,
}

#[derive(Serialize)]
pub struct CreateAccountResetInvitationRequest {
    pub username: Option<JidNode>,
    #[serde(rename = "ttl")]
    pub ttl_secs: Option<u32>,
    pub additional_data: serde_json::Value,
}

// NOTE: Some fields might be optional. Check before uncommenting.
#[derive(Deserialize)]
pub struct InviteInfo {
    pub id: InvitationId,
    // pub r#type: String,
    // pub reusable: bool,
    // pub inviter: BareJid,
    pub jid: BareJid,
    // pub uri: String,
    // pub landing_page: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires: DateTime<Utc>,
    // #[serde(default)]
    // pub groups: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    // pub source: Option<String>,
    // pub reset: bool,
    // pub note: Option<String>,
    pub additional_data: serde_json::Value,
}

// MARK: - Boilerplate

impl From<UserInfo> for Member {
    fn from(info: UserInfo) -> Self {
        let role: ProsodyRoleName = info.role.expect("Members should have roles");

        Self {
            jid: info.jid,
            role: MemberRole::try_from(&role).expect("Members should have supported roles"),
        }
    }
}
