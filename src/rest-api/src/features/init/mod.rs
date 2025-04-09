// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod init_first_account;
mod init_server_config;
mod init_workspace;

use axum::routing::put;
use axum::routing::MethodRouter;

use crate::AppState;

pub use self::init_first_account::*;
pub use self::init_server_config::*;
pub use self::init_workspace::*;

pub const SERVER_CONFIG_ROUTE: &'static str = "/v1/server/config";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/init/first-account",
            MethodRouter::new()
                .put(init_first_account_route)
                .head(is_first_account_created_route),
        )
        .route(SERVER_CONFIG_ROUTE, put(init_server_config_route))
        .with_state(app_state)
}
