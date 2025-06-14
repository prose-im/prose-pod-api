// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    auth::IsAdmin,
    pod_config::PodConfigRepository,
    server_config::{errors::ServerConfigNotInitialized, server_config_controller},
};

use crate::{features::pod_config::PodAddressNotInitialized, guards::prelude::*};

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

        let AppState { db, app_config, .. } = state;

        let server_config = server_config_controller::get_server_config(db, app_config, &is_admin)
            .await?
            .ok_or(ServerConfigNotInitialized)?;

        let Some(pod_config) = PodConfigRepository::get(db).await? else {
            // NOTE: We return `PodAddressNotInitialized` and not `PodConfigNotInitialized`
            //   because we only read `.pod_address()` and initializing the address initializes
            //   the whole config.
            return Err(Error::from(PodAddressNotInitialized));
        };
        let pod_address = (pod_config.network_address()).ok_or(PodAddressNotInitialized)?;

        Ok(Self {
            server_domain: server_config.domain,
            pod_address,
            federation_enabled: server_config.federation_enabled,
        })
    }
}
