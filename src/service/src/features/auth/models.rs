// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::BareJid;
use secrecy::SecretString;
use serdev::Serialize;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::{
    auth::util::random_secret_url_safe, members::MemberRole, models::SerializableSecretString,
};

pub const PASSWORD_RESET_TOKEN_LENGTH: usize = 36;

pub struct Credentials {
    pub jid: BareJid,
    pub password: SecretString,
}

/// An OAuth 2.0 token (provided by Prosody).
#[derive(Debug)]
#[repr(transparent)]
pub struct AuthToken(pub SecretString);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "test", derive(serdev::Serialize, serdev::Deserialize))]
pub struct UserInfo {
    pub jid: BareJid,
    pub role: MemberRole,
}

impl UserInfo {
    pub fn is(&self, other: &BareJid) -> bool {
        &self.jid == other
    }
    pub fn is_admin(&self) -> bool {
        self.role == MemberRole::Admin
    }
}

/// Ensures a user is logged in.
// NOTE: Has a private field to ensure it cannot be created from somewhere else.
#[repr(transparent)]
pub struct Authenticated(());

impl From<UserInfo> for Authenticated {
    fn from(_: UserInfo) -> Self {
        Self(())
    }
}

/// Ensures the logged in user is an admin.
///
/// It's not perfect, one day we'll replace it with scopes and permissions,
/// but it'll do for now.
pub struct IsAdmin;

#[derive(Debug, Clone)]
#[derive(serdev::Deserialize)]
#[serde(validate = "Validate::validate")]
#[repr(transparent)]
pub struct PasswordResetToken(SerializableSecretString);

impl PasswordResetToken {
    pub fn new() -> Self {
        // NOTE: `random_secret`
        Self::from(random_secret_url_safe(PASSWORD_RESET_TOKEN_LENGTH))
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct PasswordResetKvRecord {
    pub key: PasswordResetToken,
    pub value: PasswordResetRecord,
}

/// The JSON value stored in the global key/value store.
#[derive(Serialize, serdev::Deserialize)]
#[derive(sea_orm::FromJsonQueryResult)]
pub struct PasswordResetRecord {
    pub jid: BareJid,
    pub expires_at: DateTime<Utc>,
}

// MARK: Validation

impl Validate for PasswordResetToken {
    fn validate(&self) -> Result<(), ValidationErrors> {
        use std::borrow::Cow;

        let mut errors = ValidationErrors::new();

        if self.0.len() != PASSWORD_RESET_TOKEN_LENGTH {
            errors.add(
                "__all__",
                ValidationError::new("length").with_message(Cow::Owned(format!(
                    "Invalid confirmation code: Expected length is {PASSWORD_RESET_TOKEN_LENGTH}."
                ))),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// MARK: BOILERPLATE

impl std::ops::Deref for AuthToken {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for PasswordResetToken {
    type Target = SerializableSecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for PasswordResetToken
where
    T: Into<SerializableSecretString>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

crate::sea_orm_string!(PasswordResetToken; secret);
