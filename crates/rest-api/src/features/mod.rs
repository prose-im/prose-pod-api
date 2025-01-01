// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
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

pub(super) fn routes() -> Vec<rocket::Route> {
    vec![
        api_docs::routes(),
        auth::routes(),
        dns_setup::routes(),
        init::routes(),
        invitations::routes(),
        members::routes(),
        network_checks::routes(),
        pod_config::routes(),
        profile::routes(),
        roles::routes(),
        server_config::routes(),
        workspace_details::routes(),
    ]
    .concat()
}
