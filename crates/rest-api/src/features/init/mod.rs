// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod init_first_account;
mod init_server_config;
mod init_workspace;

use axum::routing::put;

pub use self::init_first_account::*;
pub use self::init_server_config::*;
pub use self::init_workspace::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        init_first_account_route,
        init_server_config_route,
        init_workspace_route,
    ]
}

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/v1/init/first-account", put(init_first_account_route_axum))
        .route("/v1/server/config", put(init_server_config_route_axum))
        .route("/v1/workspace", put(init_workspace_route_axum))
}
