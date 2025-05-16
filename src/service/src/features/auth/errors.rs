// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
#[error("Cannot give a role you don't have.")]
pub struct CannotAssignRole;
