// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    server_config::entities::server_config,
    xmpp::{ServerCtl, ServerManager},
};

use super::prelude::*;

/// WARN: Use only in initialization routes! Otherwise use `ServerManager` directly.
pub struct UnauthenticatedServerManager(pub(super) ServerManager);

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedServerManager {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let app_config = try_outcome!(request_state!(req, service::AppConfig));
        let server_ctl = try_outcome!(request_state!(req, ServerCtl));
        let server_config = try_outcome!(server_config::Model::from_request(req).await);

        Outcome::Success(Self(ServerManager::new(
            Arc::new(db.clone()),
            Arc::new(app_config.clone()),
            Arc::new(server_ctl.clone()),
            server_config,
        )))
    }
}
