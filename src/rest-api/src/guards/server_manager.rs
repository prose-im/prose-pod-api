// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::{errors::ServerConfigNotInitialized, ServerConfigRepository};

use super::prelude::*;

impl FromRequestParts<AppState> for service::xmpp::ServerManager {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::extract::server_manager", level = "trace", skip_all, err)]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_config =
            (ServerConfigRepository::get(&state.db).await)?.ok_or(ServerConfigNotInitialized)?;

        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.app_config.clone()),
            Arc::new(state.server_ctl.clone()),
            server_config,
        ))
    }
}
