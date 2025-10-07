// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod invitation_controller;
pub mod invitation_repository;
pub mod invitation_service;
pub mod models;
mod workspace_invitation_notification;

pub use self::invitation_repository::*;
pub use self::invitation_service::InvitationService;
pub use self::models::*;

pub mod errors {
    use super::models::InvitationId;

    #[derive(Debug, thiserror::Error)]
    #[repr(transparent)]
    #[error("No invitation with id '{0:?}'.")]
    pub struct InvitationNotFound(pub InvitationId);

    #[derive(Debug, thiserror::Error)]
    #[error("Invitation not found.")]
    pub struct InvitationNotFoundForToken;

    #[derive(Debug, thiserror::Error)]
    #[repr(transparent)]
    #[error("Member '{0}' already exists.")]
    pub struct MemberAlreadyExists(pub String);

    #[derive(Debug, thiserror::Error)]
    #[error("Invitation already exists (choose a different username).")]
    pub struct InvitationAlreadyExists;

    #[derive(Debug, thiserror::Error)]
    #[error("Username already taken.")]
    pub struct UsernameAlreadyTaken;
}
