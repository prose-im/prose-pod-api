// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use hmac::{Hmac, Mac};
use jwt::{SignWithKey as _, VerifyWithKey as _};
use prose_xmpp::BareJid;
use secrecy::Secret;
use sha2::Sha256;
use std::{collections::BTreeMap, env};
use xmpp_parsers::jid;

use super::auth_service::JWT_PROSODY_TOKEN_KEY;

const ENV_JWT_SIGNING_KEY: &'static str = "JWT_SIGNING_KEY";
pub const JWT_JID_KEY: &'static str = "jid";

#[derive(Debug, Clone)]
pub struct JWTService {
    jwt_key: JWTKey,
}

impl JWTService {
    pub fn new(jwt_key: JWTKey) -> Self {
        Self { jwt_key }
    }

    pub fn from_env() -> Result<Self, JWTError> {
        let jwt_key = JWTKey::from_env()?;
        Ok(Self { jwt_key })
    }

    pub fn generate_jwt(
        &self,
        jid: &BareJid,
        add_claims: impl FnOnce(&mut BTreeMap<&str, String>) -> (),
    ) -> Result<String, JWTError> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;

        let mut claims = BTreeMap::new();
        claims.insert(JWT_JID_KEY, jid.to_string());
        add_claims(&mut claims);
        claims
            .sign_with_key(&jwt_key)
            .map_err(Arc::new)
            .map_err(JWTError::Sign)
    }

    pub fn verify(&self, jwt: &str) -> Result<BTreeMap<String, String>, JWTError> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;
        jwt.verify_with_key(&jwt_key)
            .map_err(Arc::new)
            .map_err(JWTError::Verify)
    }
}

pub type Error = JWTError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum JWTError {
    #[error("Could not sign JWT claims: {0}")]
    Sign(Arc<jwt::Error>),
    #[error("Could not verify JWT claims: {0}")]
    Verify(Arc<jwt::Error>),
    #[error("{0}")]
    InvalidClaim(#[from] InvalidJwtClaimError),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidJwtClaimError {
    #[error("The provided JWT does not contain a '{0}' claim")]
    MissingClaim(String),
    #[error("The JID present in the JWT could not be parsed to a valid JID: {0}")]
    InvalidJid(#[from] Arc<jid::Error>),
}

impl From<jid::Error> for InvalidJwtClaimError {
    fn from(err: jid::Error) -> Self {
        Self::InvalidJid(Arc::new(err))
    }
}

pub struct JWT {
    pub claims: BTreeMap<String, String>,
}

impl JWT {
    pub fn try_from(jwt: &str, jwt_service: &JWTService) -> Result<Self, JWTError> {
        jwt_service.verify(jwt).map(|claims| Self { claims })
    }
}

impl JWT {
    pub fn jid(&self) -> Result<BareJid, InvalidJwtClaimError> {
        let Some(jid) = self.claims.get(JWT_JID_KEY) else {
            return Err(InvalidJwtClaimError::MissingClaim(JWT_JID_KEY.to_owned()));
        };
        let jid = BareJid::new(jid.as_str())?;
        Ok(jid)
    }
    pub fn prosody_token(&self) -> Result<Secret<String>, InvalidJwtClaimError> {
        let Some(token) = self.claims.get(JWT_PROSODY_TOKEN_KEY) else {
            return Err(InvalidJwtClaimError::MissingClaim(
                JWT_PROSODY_TOKEN_KEY.to_owned(),
            ));
        };
        Ok(token.to_owned().into())
    }
}

#[derive(Debug, Clone)]
pub struct JWTKey(String);

impl JWTKey {
    pub fn from_env() -> Result<Self, JWTError> {
        match env::var(ENV_JWT_SIGNING_KEY) {
            Ok(v) => Ok(Self(v)),
            Err(e) => Err(JWTError::Other(format!(
                "Environment variable '{ENV_JWT_SIGNING_KEY}' not found: {e}",
            ))),
        }
    }

    /// WARN: Do not use this in production!
    #[cfg(debug_assertions)]
    pub fn custom(key: &'static str) -> Self {
        Self(key.to_string())
    }
}

impl JWTKey {
    pub fn as_hmac_sha_256(&self) -> Result<Hmac<Sha256>, JWTError> {
        Hmac::new_from_slice(self.0.as_bytes()).map_err(|e| {
            JWTError::Other(format!(
                "Cannot map `{}` to `{}`: {e}",
                stringify!(JWTKey),
                stringify!(Hmac<Sha256>),
            ))
        })
    }
}
