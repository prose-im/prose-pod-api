// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

#[derive(Debug, thiserror::Error)]
#[error("Invalid credentials.")]
pub struct InvalidCredentials;

#[derive(Debug, thiserror::Error)]
#[error("Invalid auth token.")]
pub struct InvalidAuthToken;

#[derive(Debug, thiserror::Error)]
#[error("Cannot change your own role.")]
pub struct CannotChangeOwnRole;

#[derive(Debug, thiserror::Error)]
#[error("Cannot give a role you don’t have.")]
pub struct CannotAssignRole;

#[derive(Debug, thiserror::Error)]
#[error("Cannot reset someone else’s password (unless you’re an admin).")]
pub struct CannotResetPassword;

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("Missing email address for {jid}.", jid = 0.to_string())]
pub struct MissingEmailAddress(pub BareJid);

#[derive(Debug, thiserror::Error)]
#[error("Token expired.")]
pub struct PasswordResetTokenExpired;
