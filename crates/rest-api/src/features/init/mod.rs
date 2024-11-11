// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod init_first_account;
mod init_server_config;
mod init_workspace;

pub use init_first_account::*;
pub use init_server_config::*;
pub use init_workspace::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        init_first_account_route,
        init_server_config_route,
        init_workspace_route,
    ]
}
