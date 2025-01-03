// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod startup_actions;

pub mod api_docs;
pub mod auth;
pub mod dns_setup;
pub mod init;
pub mod invitations;
pub mod members;
pub mod network_checks;
pub mod pod_config;
pub mod profile;
pub mod roles;
pub mod server_config;
pub mod workspace_details;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(api_docs::router())
        .merge(auth::router())
        .merge(dns_setup::router())
        .merge(init::router())
        .merge(invitations::router())
        .merge(members::router())
        .merge(network_checks::router())
        .merge(pod_config::router())
        .merge(profile::router())
        .merge(roles::router())
        .merge(server_config::router())
        .merge(workspace_details::router())
}
