// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serdev::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::members::MemberRole;
use crate::models::{BareJid, EmailAddress, SerializableSecretString};
use crate::xmpp::JidNode;

pub use super::workspace_invitation_notification::WorkspaceInvitationPayload;

// MARK: Invitation

#[derive(Debug, Clone)]
pub struct Invitation {
    pub id: InvitationId,
    pub created_at: OffsetDateTime,
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub email_address: EmailAddress,
    /// Expiring one-time use token used to accept an invitation.
    /// Will change every time an admin resends the invitation.
    /// Will be deleted along with the entire invitation once used.
    pub accept_token: InvitationToken,
    pub accept_token_expires_at: OffsetDateTime,
    /// Unique token used by someone to reject an invitation (e.g. because of
    /// misspelled email address).
    /// Never expires, will be usable as long as the invitation still exists.
    /// Will be deleted along with the entire invitation once used.
    pub reject_token: InvitationToken,
}

impl Invitation {
    pub fn contact(&self) -> InvitationContact {
        InvitationContact::Email {
            email_address: self.email_address.clone(),
        }
    }
    pub fn is_expired(&self) -> bool {
        self.accept_token_expires_at < OffsetDateTime::now_utc()
    }
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct WorkspaceInvitationBasicDetails {
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub is_expired: bool,
}

// MARK: Invitation ID

pub type InvitationId = InvitationToken;

// MARK: Invitation token

#[derive(Clone)]
#[derive(Serialize, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
#[repr(transparent)]
pub struct InvitationToken(SerializableSecretString);

// NOTE: Useful for logging purposes because `InvitationId` is an alias for
//   `InvitationToken` (because of how it works in Prosody). This way we can
//   follow the life of an invitation without leaking the accept token.
impl std::fmt::Debug for InvitationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use secrecy::ExposeSecret as _;
        use std::hash::{DefaultHasher, Hash, Hasher as _};

        let mut hasher = DefaultHasher::new();
        self.expose_secret().hash(&mut hasher);

        write!(f, "{:x}", hasher.finish())
    }
}

impl InvitationToken {
    fn validate(&self) -> anyhow::Result<()> {
        const MAX_LENGTH: usize = 256;

        if self.0.len() <= MAX_LENGTH {
            Ok(())
        } else {
            Err(anyhow::Error::msg(
                "Invalid invitation token: Max length is {MAX_LENGTH}.",
            ))
        }
    }
}

// MARK: Forms

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// NOTE: No need to validate as `EmailAddress` is parsed.
#[serde(tag = "channel", rename_all = "snake_case")]
pub enum InvitationContact {
    Email { email_address: EmailAddress },
}

// WARN: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub enum InvitationChannel {
    Email,
}

#[derive(Debug)]
pub struct InviteMemberForm {
    pub username: JidNode,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
}

// MARK: - Boilerplate

impl std::ops::Deref for InvitationToken {
    type Target = <SerializableSecretString as std::ops::Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl From<InvitationToken> for secrecy::SecretString {
    fn from(token: InvitationToken) -> Self {
        token.0.into()
    }
}

impl From<secrecy::SecretString> for InvitationToken {
    fn from(secret: secrecy::SecretString) -> Self {
        Self(secret.into())
    }
}

#[cfg(feature = "test")]
impl std::cmp::PartialEq for InvitationToken {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(feature = "test")]
impl std::cmp::Eq for InvitationToken {}

#[cfg(feature = "test")]
impl std::hash::Hash for InvitationToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret().hash(state);
    }
}

impl From<Invitation> for WorkspaceInvitationBasicDetails {
    fn from(invitation: Invitation) -> Self {
        Self {
            is_expired: invitation.is_expired(),
            jid: invitation.jid,
            pre_assigned_role: invitation.pre_assigned_role,
        }
    }
}
