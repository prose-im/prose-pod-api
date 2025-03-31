// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{pod_config::PodConfigRepository, server_config::ServerConfig};

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
        let server_domain = ServerConfig::from_request_parts(parts, state).await?.domain;

        let Some(pod_config) = PodConfigRepository::get(&state.db).await? else {
            // NOTE: We return `PodAddressNotInitialized` and not `PodConfigNotInitialized`
            //   because we only read `.pod_address()` and initializing the address initializes
            //   the whole config.
            return Err(Error::from(PodAddressNotInitialized));
        };
        let pod_address =
            (pod_config.pod_address()).ok_or(Error::from(PodAddressNotInitialized))?;

        Ok(Self {
            server_domain,
            pod_address,
        })
    }
}
