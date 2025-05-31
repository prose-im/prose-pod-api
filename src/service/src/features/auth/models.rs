use chrono::{DateTime, Utc};
use jid::BareJid;
use secrecy::SecretString;

use crate::{app_config::ConfigAuth, models::SerializableSecretString};

pub struct Credentials {
    pub jid: BareJid,
    pub password: SecretString,
}

/// Ensures a user is logged in.
pub struct Authenticated;

/// Ensures the logged in user is an admin.
///
/// It's not perfect, one day we'll replace it with scopes and permissions,
/// but it'll do for now.
pub struct IsAdmin;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct PasswordResetRecord {
    token: SerializableSecretString,
    expires_at: DateTime<Utc>,
}

impl PasswordResetRecord {
    pub(super) fn new(token: SecretString, auth_config: &ConfigAuth) -> Self {
        let password_reset_token_ttl = (auth_config.password_reset_token_ttl.to_std()).expect(
            "`app_config.auth.password_reset_token_ttl` contains years or months. Not supported.",
        );

        Self {
            token: SerializableSecretString::from(token),
            expires_at: Utc::now() + password_reset_token_ttl,
        }
    }
}
