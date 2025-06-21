// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod models;
pub mod workspace_controller;
pub mod workspace_service;

pub use models::*;
pub use workspace_service::*;

pub mod errors {
    #[derive(Debug, thiserror::Error)]
    pub enum WorkspaceNotInitialized {
        #[error("Workspace not initialized: {0}")]
        WithReason(&'static str),
        #[error("Workspace not initialized")]
        NoReason,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Workspace already initialized.")]
    pub struct WorkspaceAlreadyInitialized;
}
