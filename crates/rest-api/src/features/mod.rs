// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod startup_actions;

pub mod api_docs;
pub mod auth;
pub mod backups;
pub mod dns_setup;
pub mod init;
pub mod invitations;
pub mod members;
pub mod mfa;
pub mod network_checks;
pub mod pod_config;
pub mod profile;
pub mod reactions;
pub mod roles;
pub mod server_config;
pub mod workspace_details;

pub(super) fn routes() -> Vec<rocket::Route> {
    vec![
        api_docs::routes(),
        auth::routes(),
        backups::routes(),
        dns_setup::routes(),
        init::routes(),
        invitations::routes(),
        members::routes(),
        mfa::routes(),
        network_checks::routes(),
        pod_config::routes(),
        profile::routes(),
        reactions::routes(),
        roles::routes(),
        server_config::routes(),
        workspace_details::routes(),
    ]
    .concat()
}

pub(super) fn router() -> axum::Router {
    axum::Router::new()
        .merge(api_docs::router())
        .merge(auth::router())
        .merge(backups::router())
        .merge(dns_setup::router())
        .merge(init::router())
        .merge(invitations::router())
        .merge(members::router())
        .merge(mfa::router())
        .merge(network_checks::router())
        .merge(pod_config::router())
        .merge(profile::router())
        .merge(reactions::router())
        .merge(roles::router())
        .merge(server_config::router())
        .merge(workspace_details::router())
}
