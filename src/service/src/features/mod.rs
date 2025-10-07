// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod app_config;
pub mod auth;
pub mod cron;
pub mod factory_reset;
pub mod global_storage;
pub mod identity_provider;
pub mod invitations;
pub mod licensing;
pub mod members;
pub mod network_checks;
pub mod notifications;
pub mod onboarding;
pub mod pod_version;
pub mod prose_pod_server_api;
pub mod prose_pod_server_service;
pub mod prosody;
pub mod secrets_store;
pub mod server_config;
pub mod workspace;
pub mod xmpp;

pub mod init {
    pub mod errors {
        #[derive(Debug, thiserror::Error)]
        #[error("First account already created.")]
        pub struct FirstAccountAlreadyCreated;
    }
}
