// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use secrecy::SecretString;
#[cfg(debug_assertions)]
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{models::BareJid, util::either::Either};

use super::errors::{InvalidAuthToken, InvalidCredentials};

#[derive(Debug, Clone)]
pub struct AuthService {
    implem: Arc<dyn AuthServiceImpl>,
}

impl AuthService {
    pub fn new(implem: Arc<dyn AuthServiceImpl>) -> Self {
        Self { implem }
    }
}

impl AuthService {
    // TODO: Allow passing `Credentials`.
    /// Generates a token from a username and password.
    #[instrument(level = "trace", skip_all, fields(jid = jid.to_string()))]
    pub async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
        self.implem.log_in(jid, password).await
    }
    #[instrument(level = "trace", skip_all)]
    pub async fn get_user_info(
        &self,
        token: AuthToken,
    ) -> Result<UserInfo, Either<InvalidAuthToken, anyhow::Error>> {
        self.implem.get_user_info(token).await
    }
    #[instrument(level = "trace", skip_all, err)]
    pub async fn register_oauth2_client(&self) -> Result<(), anyhow::Error> {
        self.implem.register_oauth2_client().await
    }
}

/// An OAuth 2.0 token (provided by Prosody).
#[derive(Debug)]
pub struct AuthToken(pub SecretString);

impl Deref for AuthToken {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
#[cfg_attr(debug_assertions, derive(Serialize, Deserialize))]
pub struct UserInfo {
    pub jid: BareJid,
}

#[async_trait::async_trait]
pub trait AuthServiceImpl: Debug + Sync + Send {
    async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>>;
    async fn get_user_info(
        &self,
        token: AuthToken,
    ) -> Result<UserInfo, Either<InvalidAuthToken, anyhow::Error>>;
    async fn register_oauth2_client(&self) -> Result<(), anyhow::Error>;
}
