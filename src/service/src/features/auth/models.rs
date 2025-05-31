// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use anyhow::Context;
use chrono::{DateTime, Utc};
use jid::BareJid;
use secrecy::SecretString;

use crate::{members::MemberRole, models::SerializableSecretString};

pub struct Credentials {
    pub jid: BareJid,
    pub password: SecretString,
}

/// An OAuth 2.0 token (provided by Prosody).
#[derive(Debug)]
pub struct AuthToken(pub SecretString);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct PasswordResetToken(pub SerializableSecretString);

pub(crate) struct PasswordResetRecord {
    pub jid: BareJid,
    pub token: PasswordResetToken,
    pub expires_at: DateTime<Utc>,
}

/// A [`PasswordResetRecord`], but serializable to JSON to store in the global
/// key/value store.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PasswordResetKvRecord {
    pub key: BareJid,
    pub value: PasswordResetRecordData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PasswordResetRecordData {
    pub token: PasswordResetToken,
    pub expires_at: DateTime<Utc>,
}

// MARK: BOILERPLATE

impl Deref for AuthToken {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PasswordResetToken {
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

impl Into<SecretString> for PasswordResetToken {
    fn into(self) -> SecretString {
        self.0.into_secret_string()
    }
}

impl From<PasswordResetKvRecord> for PasswordResetRecord {
    fn from(record: PasswordResetKvRecord) -> Self {
        Self {
            jid: record.key,
            token: record.value.token,
            expires_at: record.value.expires_at,
        }
    }
}

impl From<PasswordResetRecord> for PasswordResetKvRecord {
    fn from(record: PasswordResetRecord) -> Self {
        Self {
            key: record.jid,
            value: PasswordResetRecordData {
                token: record.token,
                expires_at: record.expires_at,
            },
        }
    }
}

impl TryFrom<crate::global_storage::KvRecord> for PasswordResetKvRecord {
    type Error = anyhow::Error;

    fn try_from(record: crate::global_storage::KvRecord) -> Result<Self, Self::Error> {
        let key = BareJid::from_str(&record.key).context("Invalid key")?;
        let value = serde_json::from_value::<PasswordResetRecordData>(record.value)
            .context("Invalid value")?;
        Ok(Self { key, value })
    }
}
