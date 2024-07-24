// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    env,
    ops::Deref,
    time::{Duration, SystemTime},
};

use josekit::{
    jwe::{Dir, JweHeader},
    jwt::{self, JwtPayload},
    JoseError,
};
use prose_xmpp::BareJid;
use secrecy::{ExposeSecret as _, SecretString};
use xmpp_parsers::jid;

use super::auth_service::JWT_PROSODY_TOKEN_KEY;

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
        add_claims: impl FnOnce(&mut JwtPayload) -> Result<(), JoseError>,
    ) -> Result<SecretString, JWTError> {
        let mut header = JweHeader::new();
        header.set_token_type("JWT");
        // NOTE: AES GCM encryption using a 256-bit (32 bytes) key
        header.set_content_encryption("A256GCM");

        let mut payload = JwtPayload::new();
        payload.set_issuer("https://prose.org");
        payload.set_subject(jid.to_string());
        let now = SystemTime::now();
        payload.set_issued_at(&now);
        // TTL = 3 hours
        let expires_at = now + Duration::from_secs(chrono::Duration::hours(3).num_seconds() as u64);
        payload.set_expires_at(&expires_at);
        add_claims(&mut payload).map_err(JWTError::CouldNotAddClaims)?;

        let encrypter = Dir
            .encrypter_from_bytes(self.jwt_key.expose_secret())
            .map_err(JWTError::InvalidJwtKey)?;
        let jwt =
            jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(JWTError::Encode)?;

        Ok(SecretString::new(jwt))
    }

    pub fn verify(&self, jwt: &SecretString) -> Result<(JwtPayload, JweHeader), JWTError> {
        let decrypter = Dir
            .decrypter_from_bytes(self.jwt_key.expose_secret())
            .map_err(JWTError::InvalidJwtKey)?;
        let (payload, header) = jwt::decode_with_decrypter(&jwt.expose_secret(), &decrypter)
            .map_err(JWTError::Decode)?;
        Ok((payload, header))
    }
}

pub type Error = JWTError;

#[derive(Debug, thiserror::Error)]
pub enum JWTError {
    #[error("Invalid JWT key: {0}")]
    InvalidJwtKey(JoseError),
    #[error("Could not add custom claims to the JWT: {0}")]
    CouldNotAddClaims(JoseError),
    #[error("Could not encode JWT: {0}")]
    Encode(JoseError),
    #[error("Could not decode JWT: {0}")]
    Decode(JoseError),
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
    pub header: JweHeader,
    pub payload: JwtPayload,
}

impl JWT {
    pub fn try_from(jwt: &SecretString, jwt_service: &JWTService) -> Result<Self, JWTError> {
        jwt_service
            .verify(jwt)
            .map(|(payload, header)| Self { payload, header })
    }
}

impl JWT {
    fn string_claim<'a>(&'a self, key: &str) -> Result<&'a str, InvalidJwtClaimError> {
        let Some(value) = self.payload.claim(key) else {
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
pub struct JWTKey(SecretString);

impl JWTKey {
    pub fn from_env() -> Result<Self, JWTError> {
        match env::var(ENV_JWT_SIGNING_KEY) {
            Ok(v) => Ok(Self(v.into())),
            Err(e) => Err(JWTError::Other(format!(
                "Environment variable '{ENV_JWT_SIGNING_KEY}' not found: {e}",
            ))),
        }
    }

    /// WARN: Do not use this in production!
    #[cfg(debug_assertions)]
    pub fn custom(key: &'static str) -> Self {
        Self(key.to_owned().into())
    }
}

impl Deref for JWTKey {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
