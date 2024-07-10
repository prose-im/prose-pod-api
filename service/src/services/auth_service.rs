// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use prose_xmpp::BareJid;

use crate::{
    prosody::{ProsodyOAuth2, ProsodyOAuth2Error},
    services::jwt_service::{JWTError, JWTService},
};

use super::jwt_service::JWT;

pub const JWT_PROSODY_TOKEN_KEY: &'static str = "prosody_token";

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

pub trait AuthServiceImpl: Debug + Sync + Send {
    /// Generates a token from a username and password.
    fn log_in(&self, jid: &BareJid, password: &str) -> Result<String, AuthError>;
    fn verify(&self, jwt: &str) -> Result<JWT, JWTError>;
}

#[derive(Debug, Clone)]
pub struct LiveAuthService {
    jwt_service: JWTService,
    prosody_oauth2: ProsodyOAuth2,
}

impl LiveAuthService {
    pub fn new(jwt_service: JWTService, prosody_oauth2: ProsodyOAuth2) -> Self {
        Self {
            jwt_service,
            prosody_oauth2,
        }
    }
}

impl AuthServiceImpl for LiveAuthService {
    fn log_in(&self, jid: &BareJid, password: &str) -> Result<String, AuthError> {
        let Some(prosody_token) = self.prosody_oauth2.log_in(jid, password)? else {
            Err(AuthError::InvalidCredentials)?
        };

        let token = self.jwt_service.generate_jwt(jid, |claims| {
            // TODO: Do not store this in the JWT (potential security issue?)
            claims.insert(JWT_PROSODY_TOKEN_KEY, prosody_token);
        })?;

        Ok(token)
    }
    fn verify(&self, jwt: &str) -> Result<JWT, JWTError> {
        JWT::try_from(jwt, &self.jwt_service)
    }
}

pub type Error = AuthError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("`{}` error: {0}", stringify!(ProsodyOAuth2))]
    ProsodyOAuth2Err(#[from] ProsodyOAuth2Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("`{}` error: {0}", stringify!(JWTService))]
    JWTErr(#[from] JWTError),
    #[cfg(debug_assertions)]
    #[error("{0}")]
    Other(String),
}
