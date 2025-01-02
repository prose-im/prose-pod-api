// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{server_config::entities::server_config, xmpp::ServerManager};

use super::{prelude::*, UnauthenticatedServerManager};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerManager {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        try_outcome!(check_caller_is_admin(req, None).await);

        UnauthenticatedServerManager::from_request(req)
            .await
            .map(|m| m.0)
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for ServerManager {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_config = server_config::Model::from_request_parts(parts, state).await?;

        Ok(ServerManager::new(
            Arc::new(state.db.clone()),
            Arc::new(state.app_config.clone()),
            Arc::new(state.server_ctl.clone()),
            server_config,
        ))
    }
}
