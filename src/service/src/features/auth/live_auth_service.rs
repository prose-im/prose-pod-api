// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::anyhow;
use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;

use crate::{models::BareJid, prosody::ProsodyOAuth2, util::Either};

use super::{
    auth_service::{AuthToken, UserInfo},
    errors::InvalidCredentials,
    AuthError, AuthServiceImpl,
};

#[derive(Debug, Clone)]
pub struct LiveAuthService {
    prosody_oauth2: Arc<ProsodyOAuth2>,
}

impl LiveAuthService {
    pub fn new(prosody_oauth2: Arc<ProsodyOAuth2>) -> Self {
        Self { prosody_oauth2 }
    }
}

#[async_trait::async_trait]
impl AuthServiceImpl for LiveAuthService {
    async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
        match self.prosody_oauth2.log_in(jid, password).await {
            Ok(Some(token)) => Ok(AuthToken(token.into())),
            Ok(None) => Err(Either::E1(InvalidCredentials)),
            Err(err) => Err(Either::E2(
                anyhow!(err).context("Prosody OAuth 2.0 error"),
            )),
        }
    }

    async fn get_user_info(&self, token: AuthToken) -> Result<UserInfo, AuthError> {
        let response = self
            .prosody_oauth2
            .call(
                |client| {
                    client
                        .get(self.prosody_oauth2.url("userinfo"))
                        .bearer_auth(token.expose_secret())
                },
                |res| res.status.is_success(),
            )
            .await?;

        let body = response.text();
        let res: UserInfoResponse = serde_json::from_str(&body)?;

        Ok(UserInfo::from(res))
    }

    async fn register_oauth2_client(&self) -> Result<(), AuthError> {
        self.prosody_oauth2
            .register()
            .await
            .map_err(AuthError::from)
    }
}

/// Example value:
///
/// ```json
/// {
///   "iss":"http://prose-pod-server:5280"
///   "sub":"xmpp:alice@test.org"
/// }
/// ```
#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    sub: String,
}

impl From<UserInfoResponse> for UserInfo {
    fn from(res: UserInfoResponse) -> Self {
        let jid_str = res.sub.strip_prefix("xmpp:").unwrap();
        // NOTE: This JID is returned by prosody so we can assume it's well formatted.
        let jid = BareJid::new(jid_str).unwrap();
        Self { jid }
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        Self::Other(format!("reqwest::Error: {err}"))
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(format!("serde_json::Error: {err}"))
    }
}
