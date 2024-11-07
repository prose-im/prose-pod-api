// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::env;

use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey as _, VerifyWithKey as _};
use secrecy::{ExposeSecret as _, SecretString};
use sha2::Sha256;

use crate::{
    features::auth::JWT_PROSODY_TOKEN_KEY,
    models::xmpp::{jid, BareJid},
};

const ENV_JWT_SIGNING_KEY: &'static str = "JWT_SIGNING_KEY";
pub const JWT_JID_KEY: &'static str = "sub";

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
        add_claims: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>) -> (),
    ) -> Result<SecretString, JWTError> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;

        let mut claims = serde_json::Map::new();
        let now = Utc::now();
        claims.insert("iss".into(), "https://prose.org".into());
        claims.insert(JWT_JID_KEY.into(), jid.to_string().into());
        claims.insert("iat".into(), (now.timestamp() as usize).into());
        claims.insert(
            "exp".into(),
            ((now + Duration::hours(3)).timestamp() as usize).into(),
        );
        add_claims(&mut claims);
        let jwt = claims.sign_with_key(&jwt_key).map_err(JWTError::Sign)?;
        Ok(SecretString::new(jwt))
    }

    pub fn verify(
        &self,
        jwt: &SecretString,
    ) -> Result<serde_json::Map<String, serde_json::Value>, JWTError> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;
        jwt.expose_secret()
            .verify_with_key(&jwt_key)
            .map_err(JWTError::Verify)
    }
}

pub type Error = JWTError;

#[derive(Debug, thiserror::Error)]
pub enum JWTError {
    #[error("Could not sign JWT claims: {0}")]
    Sign(jwt::Error),
    #[error("Could not verify JWT claims: {0}")]
    Verify(jwt::Error),
    #[error("{0}")]
    InvalidClaim(#[from] InvalidJwtClaimError),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidJwtClaimError {
    #[error("The provided JWT does not contain a '{0}' claim")]
    MissingClaim(String),
    #[error("Invalid '{key}' claim: {value:?}")]
    InvalidClaim {
        key: String,
        value: serde_json::Value,
    },
    #[error("The JID present in the JWT could not be parsed to a valid JID: {0}")]
    InvalidJid(#[from] jid::Error),
}

pub struct JWT {
    pub claims: serde_json::Map<String, serde_json::Value>,
}

impl JWT {
    pub fn try_from(jwt: &SecretString, jwt_service: &JWTService) -> Result<Self, JWTError> {
        jwt_service.verify(jwt).map(|claims| Self { claims })
    }
}

impl JWT {
    fn string_claim<'a>(&'a self, key: &str) -> Result<&'a str, InvalidJwtClaimError> {
        let Some(value) = self.claims.get(key) else {
            return Err(InvalidJwtClaimError::MissingClaim(key.to_owned()));
        };
        let Some(value) = value.as_str() else {
            return Err(InvalidJwtClaimError::InvalidClaim {
                key: key.to_owned(),
                value: value.to_owned(),
            });
        };
        Ok(value)
    }
    pub fn jid(&self) -> Result<BareJid, InvalidJwtClaimError> {
        let claim_value = self.string_claim(JWT_JID_KEY)?;
        let jid = BareJid::new(claim_value)?;
        Ok(jid)
    }
    pub fn prosody_token(&self) -> Result<SecretString, InvalidJwtClaimError> {
        let claim_value = self.string_claim(JWT_PROSODY_TOKEN_KEY)?;
        Ok(claim_value.to_owned().into())
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
