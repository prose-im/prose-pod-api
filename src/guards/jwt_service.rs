// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey as _, VerifyWithKey as _};
use sha2::Sha256;
use std::{collections::BTreeMap, env};

use crate::error::Error;

pub const ENV_JWT_SIGNING_KEY: &'static str = "JWT_SIGNING_KEY";
pub const JWT_JID_KEY: &'static str = "jid";

pub struct JWTService {
    jwt_key: JWTKey,
}

impl JWTService {
    pub fn new(jwt_key: JWTKey) -> Self {
        Self { jwt_key }
    }

    pub fn from_env() -> Result<Self, Error> {
        let jwt_key = JWTKey::from_env()?;
        Ok(Self { jwt_key })
    }

    pub fn generate_jwt(&self, jid: &JID) -> Result<String, Error> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;

        let mut claims = BTreeMap::new();
        claims.insert(JWT_JID_KEY, jid.to_string());
        claims
            .sign_with_key(&jwt_key)
            .map_err(|e| Error::InternalServerError {
                reason: format!("Could not sign JWT claims: {e}"),
            })
    }

    pub fn verify(&self, jwt: &str) -> Result<BTreeMap<String, String>, Error> {
        let jwt_key = self.jwt_key.as_hmac_sha_256()?;
        jwt.verify_with_key(&jwt_key)
            .map_err(|e| Error::InternalServerError {
                reason: format!("Could not verify JWT claims: {e}"),
            })
    }
}

pub struct JWTKey(String);

impl JWTKey {
    pub fn from_env() -> Result<Self, Error> {
        match env::var(ENV_JWT_SIGNING_KEY) {
            Ok(v) => Ok(Self(v)),
            Err(e) => Err(Error::InternalServerError {
                reason: format!(
                    "Environment variable '{}' not found: {}",
                    ENV_JWT_SIGNING_KEY, e
                ),
            }),
        }
    }

    /// WARN: Do not use this in production!
    pub fn custom(key: &'static str) -> Self {
        Self(key.to_string())
    }
}

impl JWTKey {
    pub fn as_hmac_sha_256(&self) -> Result<Hmac<Sha256>, Error> {
        Hmac::new_from_slice(self.0.as_bytes()).map_err(|e| Error::InternalServerError {
            reason: format!("{}", e),
        })
    }
}
