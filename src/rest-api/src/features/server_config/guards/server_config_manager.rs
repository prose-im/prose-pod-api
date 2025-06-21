// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{error::prelude::*, guards::prelude::*};

impl FromRequestParts<AppState> for service::server_config::ServerConfigManager {
    type Rejection = error::Error;

    #[tracing::instrument(
        name = "req::extract::server_config_manager",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            Arc::new(state.db.clone()),
            state.app_config.clone(),
            Arc::new(state.server_ctl.clone()),
        ))
    }
}
