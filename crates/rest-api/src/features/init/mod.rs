// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod init_first_account;
mod init_server_config;
mod init_workspace;

use axum::routing::put;

use crate::AppState;

pub use self::init_first_account::*;
pub use self::init_server_config::*;
pub use self::init_workspace::*;

pub const SERVER_CONFIG_ROUTE: &'static str = "/v1/server/config";
pub const WORKSPACE_ROUTE: &'static str = "/v1/workspace";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/init/first-account", put(init_first_account_route))
        .route(SERVER_CONFIG_ROUTE, put(init_server_config_route))
        .route(WORKSPACE_ROUTE, put(init_workspace_route))
        .with_state(app_state)
}
