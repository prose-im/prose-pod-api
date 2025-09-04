// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{auth::IsAdmin, server_config::server_config_controller};

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::network_checks::PodNetworkConfig {
    type Rejection = error::Error;

    #[tracing::instrument(
        name = "req::extract::pod_network_config",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let is_admin = IsAdmin::from_request_parts(parts, state).await?;

        let AppState { db, .. } = state;
        let ref app_config = state.app_config_frozen();

        let server_config =
            server_config_controller::get_server_config(db, app_config, &is_admin).await?;

        Ok(Self::new(app_config, server_config.federation_enabled))
    }
}
