// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

use crate::error::Error;

pub const ENV_JWT_SIGNING_KEY: &'static str = "JWT_SIGNING_KEY";

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
