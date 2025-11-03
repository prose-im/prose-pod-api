// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use secrecy::SecretString;
use serdev::Deserialize;
use time::OffsetDateTime;

use crate::members::MemberRole;

// TODO: Make it a proper newtype wrapper.
pub use secrecy::SecretString as Password;

#[derive(Debug)]
pub struct Credentials {
    pub jid: BareJid,
    pub password: Password,
}

/// An OAuth 2.0 token (provided by Prosody).
#[derive(Debug, Clone)]
#[derive(Deserialize)]
#[repr(transparent)]
pub struct AuthToken(pub SecretString);

#[derive(Debug, Clone)]
#[derive(Deserialize)]
pub struct UserInfo {
    pub jid: BareJid,
    #[serde(with = "crate::util::deserializers::prose_role_as_prosody")]
    pub primary_role: MemberRole,
}

impl UserInfo {
    pub fn is(&self, other: &BareJid) -> bool {
        &self.jid == other
    }
    pub fn is_admin(&self) -> bool {
        self.primary_role == MemberRole::Admin
    }
}

/// Ensures a user is logged in.
// NOTE: Has a private field to ensure it cannot be created from somewhere else.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct IsAdmin;

// NOTE: I (@RemiBardon) am taking a shortcut here as
//   Prosody uses invitations to do password resets,
//   therefore `PasswordResetToken` == `InvitationToken`.
pub type PasswordResetToken = crate::invitations::InvitationToken;

#[derive(Debug, Clone)]
pub struct PasswordResetRequestInfo {
    pub jid: BareJid,
    pub token: PasswordResetToken,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

impl PasswordResetRequestInfo {
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.expires_at < OffsetDateTime::now_utc()
    }
}

// MARK: - Conversions

impl From<UserInfo> for crate::members::Member {
    fn from(info: UserInfo) -> Self {
        Self {
            jid: info.jid,
            role: info.primary_role,
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for AuthToken {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AuthToken {
    pub fn into_inner(self) -> <Self as std::ops::Deref>::Target {
        self.0
    }
}

#[cfg(feature = "test")]
impl std::cmp::PartialEq for AuthToken {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(feature = "test")]
impl std::cmp::Eq for AuthToken {}

#[cfg(feature = "test")]
impl std::hash::Hash for AuthToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret().hash(state);
    }
}

#[cfg(feature = "test")]
impl From<String> for AuthToken {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
