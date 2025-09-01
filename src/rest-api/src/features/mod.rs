// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{forms::multi_value_items::rename_bracketed_query_param_names, AppState};

pub mod startup_actions;

pub mod api_docs;
pub mod auth;
pub mod dns_setup;
pub mod factory_reset;
pub mod init;
pub mod invitations;
pub mod licensing;
pub mod members;
pub mod network_checks;
pub mod onboarding;
pub mod pod_config;
pub mod profile;
pub mod reload;
pub mod server_config;
pub mod version;
pub mod workspace_details;

const NETWORK_ROUTE: &'static str = "/v1/network";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .merge(api_docs::router())
        .merge(auth::router(app_state.clone()))
        .merge(dns_setup::router(app_state.clone()))
        .merge(factory_reset::router(app_state.clone()))
        .merge(init::router(app_state.clone()))
        .merge(invitations::router(app_state.clone()))
        .merge(licensing::router(app_state.clone()))
        .merge(members::router(app_state.clone()))
        .merge(network_checks::router(app_state.clone()))
        .merge(onboarding::router(app_state.clone()))
        .merge(pod_config::router(app_state.clone()))
        .merge(profile::router(app_state.clone()))
        .merge(reload::router(app_state.clone()))
        .merge(server_config::router(app_state.clone()))
        .merge(version::router(app_state.clone()))
        .merge(workspace_details::router(app_state.clone()))
        .layer(tower::ServiceBuilder::new().map_request(rename_bracketed_query_param_names))
}
