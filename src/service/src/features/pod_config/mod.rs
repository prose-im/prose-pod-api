// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod entities;
pub(crate) mod migrations;
pub mod models;
pub mod pod_config_repository;

pub use entities::*;
pub use models::*;
pub use pod_config_repository::*;

pub mod errors {
    use super::PodConfigField;

    #[derive(Debug, thiserror::Error)]
    #[error("Pod configuration missing: {0}")]
    pub struct PodConfigMissing(pub PodConfigField);
}
