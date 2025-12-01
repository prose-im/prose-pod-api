// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod workspace_controller;
pub mod workspace_service;

pub use models::*;
pub use workspace_service::*;

pub mod models {
    use serdev::Serialize;

    use crate::models::{Avatar, Color};

    #[derive(Debug, PartialEq, Eq)]
    #[derive(Serialize)]
    #[cfg_attr(feature = "test", derive(Clone))]
    pub struct Workspace {
        pub name: String,
        pub icon: Option<Avatar>,
        pub accent_color: Option<Color>,
    }
}

pub mod errors {
    #[derive(Debug, thiserror::Error)]
    #[error("Workspace already initialized.")]
    pub struct WorkspaceAlreadyInitialized;
}
