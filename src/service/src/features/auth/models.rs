// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use chrono::{DateTime, Utc};
use jid::BareJid;
use secrecy::SecretString;
use serdev::Serialize;

use crate::{members::MemberRole, models::SerializableSecretString};

pub struct Credentials {
    pub jid: BareJid,
    pub password: SecretString,
}

/// An OAuth 2.0 token (provided by Prosody).
#[derive(Debug)]
pub struct AuthToken(pub SecretString);

#[derive(Debug)]
#[derive(serdev::Deserialize)]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
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

#[derive(Debug, Clone)]
#[derive(serdev::Deserialize)]
#[repr(transparent)]
pub struct PasswordResetToken(pub SerializableSecretString);

#[derive(serdev::Deserialize)]
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

crate::sea_orm_string!(PasswordResetToken; secret);
