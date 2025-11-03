// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use reqwest::{Client as HttpClient, StatusCode};
use secrecy::SecretString;
use serde_json::json;
use serdev::Deserialize;
use time::OffsetDateTime;
use tracing::trace;

use crate::{
    errors::Forbidden,
    invitations::{errors::*, InvitationToken},
    models::EmailAddress,
    prosody::ProsodyRoleName,
    util::either::*,
    xmpp::{jid::NodeRef, BareJid},
    AppConfig,
};

/// Rust interface to [`mod_invites_register_api`](https://github.com/prose-im/prose-pod-server/tree/master/plugins/community/mod_invites_register_api).
#[derive(Debug, Clone)]
pub struct ProsodyInvitesRegisterApi {
    http_client: HttpClient,
    api_root: String,
}

impl ProsodyInvitesRegisterApi {
    pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
        Self {
            http_client,
            api_root: format!("{}/register_api", config.server.http_url()),
        }
    }
}

impl ProsodyInvitesRegisterApi {
    pub async fn get_invite_info(
        &self,
        token: &InvitationToken,
    ) -> Result<InviteInfo, Either<InvitationNotFoundForToken, anyhow::Error>> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let request = http
            .get(format!(
                "{root}/invites/{token}",
                root = self.api_root,
                token = token.expose_secret()
            ))
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let invite_info: InviteInfo = response.json().await.context("Invalid response body")?;
            Ok(invite_info)
        } else {
            let read_body = async || {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::NOT_FOUND => Err(Either::E1(InvitationNotFoundForToken)),
                // NOTE: `403 Forbidden`s can technically happen, but it’d mean
                //   something is not configured properly internally.
                _ => Err(Either::E2(anyhow::Error::msg(format!(
                    "invites_register_api call failed: {status}: {body}",
                    body = read_body().await
                )))),
            }
        }
    }

    pub async fn register_with_invite(
        &self,
        username: Option<&NodeRef>,
        password: &SecretString,
        token: InvitationToken,
    ) -> Result<
        RegisterResponse,
        Either4<InvitationNotFoundForToken, Forbidden, MemberAlreadyExists, anyhow::Error>,
    > {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let request = http
            .put(format!("{root}/register", root = self.api_root))
            .json(&json!({
                "username": username.map(NodeRef::as_str),
                "password": password.expose_secret(),
                "token": token.expose_secret(),
            }))
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            let response: RegisterResponse =
                response.json().await.context("Invalid response body")?;
            Ok(response)
        } else {
            let read_body = async || {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::NOT_FOUND => Err(Either4::E1(InvitationNotFoundForToken)),
                StatusCode::FORBIDDEN => Err(Either4::E2(Forbidden(read_body().await))),
                StatusCode::CONFLICT => Err(Either4::E3(MemberAlreadyExists(
                    username.unwrap().to_string(),
                ))),
                _ => Err(Either4::E4(anyhow::Error::msg(format!(
                    "invites_register_api call failed: {status}: {body}",
                    body = read_body().await
                )))),
            }
        }
    }

    pub async fn reject_invite(&self, token: InvitationToken) -> Result<(), anyhow::Error> {
        use secrecy::ExposeSecret as _;

        let ref http = self.http_client;

        let request = http
            .delete(format!(
                "{root}/invites/{token}",
                root = self.api_root,
                token = token.expose_secret()
            ))
            .build()
            .context("Could not build request")?;

        trace!("Calling `{} {}`…", request.method(), request.url());
        let response = (http.execute(request).await).context("Request failed")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let read_body = async || {
                (response.text().await)
                    .ok()
                    .unwrap_or_else(|| "<empty response body>".to_owned())
            };
            match status {
                StatusCode::NOT_FOUND => Ok(()),
                // NOTE: `403 Forbidden`s can technically happen, but it’d mean
                //   something is not configured properly internally.
                _ => Err(anyhow::Error::msg(format!(
                    "invites_register_api call failed: {status}: {body}",
                    body = read_body().await
                ))),
            }
        }
    }
}

// NOTE: Some fields might be optional. Check before uncommenting.
#[derive(Deserialize)]
pub struct InviteInfo {
    // pub site_name: String,
    pub token: InvitationToken,
    // pub domain: JidDomain,
    // pub uri: String,
    // pub r#type: String,
    pub jid: BareJid,
    // pub inviter: BareJid,
    #[serde(with = "time::serde::timestamp")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    pub expires: OffsetDateTime,
    // pub reset: bool,
    #[serde(default)]
    pub additional_data: InviteAdditionalData,
}

#[derive(Deserialize, Default)]
pub struct InviteAdditionalData {
    #[serde(default)]
    pub roles: Vec<ProsodyRoleName>,
    #[serde(default)]
    pub email: Option<EmailAddress>,
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub jid: BareJid,
}

// MARK: - Boilerplate

impl TryFrom<InviteInfo> for crate::invitations::Invitation {
    type Error = anyhow::Error;

    fn try_from(invite: InviteInfo) -> Result<Self, Self::Error> {
        use crate::members::MemberRole;
        use anyhow::anyhow;
        use std::str::FromStr as _;

        let pre_assigned_role = invite
            .additional_data
            .roles
            .first()
            .map(|s| match MemberRole::from_str(s) {
                Ok(role) => Some(role),
                Err(err) => {
                    crate::util::debug_panic_or_log_warning(format!(
                        "Invalid member role '{s}': {err}"
                    ));
                    None
                }
            })
            .flatten()
            .unwrap_or_default();

        let Some(email_address) = invite.additional_data.email else {
            // NOTE: Until we implement #342, this should have been set already.
            return Err(anyhow!("Email address not stored in the invite additional data. Invite might have been created outside of Prose, which is unsupported."));
        };

        Ok(Self {
            id: invite.token.clone(),
            created_at: invite.created_at,
            jid: invite.jid,
            pre_assigned_role,
            email_address,
            accept_token: invite.token.clone(),
            accept_token_expires_at: invite.expires,
            reject_token: invite.token.clone(),
        })
    }
}
