// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    model::ServerConfig,
    services::{server_ctl::ServerCtl, server_manager::ServerManager},
};

use super::prelude::*;

/// WARN: Use only in initialization routes! Otherwise use `ServerManager` directly.
pub struct UnauthenticatedServerManager<'r>(pub(super) ServerManager<'r>);

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedServerManager<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let app_config = try_outcome!(request_state!(req, service::config::Config));
        let server_ctl = try_outcome!(request_state!(req, ServerCtl));
        let server_config = try_outcome!(ServerConfig::from_request(req).await);

        Outcome::Success(Self(ServerManager::new(
            db,
            app_config,
            server_ctl,
            server_config,
        )))
    }
}
