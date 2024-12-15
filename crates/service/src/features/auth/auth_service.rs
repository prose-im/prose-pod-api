// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use secrecy::SecretString;
#[cfg(debug_assertions)]
use serde::{Deserialize, Serialize};

use crate::{models::BareJid, prosody::ProsodyOAuth2Error};

#[derive(Debug, Clone)]
pub struct AuthService {
    implem: Arc<dyn AuthServiceImpl>,
}

impl AuthService {
    pub fn new(implem: Arc<dyn AuthServiceImpl>) -> Self {
        Self { implem }
    }
}

impl Deref for AuthService {
    type Target = Arc<dyn AuthServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
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
    /// Generates a token from a username and password.
    async fn log_in(&self, jid: &BareJid, password: &SecretString) -> Result<AuthToken, AuthError>;
    async fn get_user_info(&self, token: AuthToken) -> Result<UserInfo, AuthError>;
    async fn register_oauth2_client(&self) -> Result<(), AuthError>;
}

pub type Error = AuthError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("`{t}` error: {0}", t = stringify!(ProsodyOAuth2))]
    ProsodyOAuth2Err(#[from] ProsodyOAuth2Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("{0}")]
    Other(String),
}
