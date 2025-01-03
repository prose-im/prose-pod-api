// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::entities::server_config;

use super::prelude::*;

#[axum::async_trait]
impl FromRequestParts<AppState> for service::xmpp::ServerManager {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_config = server_config::Model::from_request_parts(parts, state).await?;

        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.app_config.clone()),
            Arc::new(state.server_ctl.clone()),
            server_config,
        ))
    }
}
