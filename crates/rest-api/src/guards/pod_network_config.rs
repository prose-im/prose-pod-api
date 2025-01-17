// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    pod_config::{PodAddress, PodConfigRepository},
    server_config::ServerConfig,
};

use crate::features::init::PodAddressNotInitialized;

use super::prelude::*;

impl FromRequestParts<AppState> for service::network_checks::PodNetworkConfig {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_domain = ServerConfig::from_request_parts(parts, state).await?.domain;

        let Some(pod_config) = PodConfigRepository::get(&state.db).await? else {
            return Err(Error::from(PodAddressNotInitialized));
        };
        let pod_address = PodAddress::try_from(pod_config)?;

        Ok(Self {
            server_domain,
            pod_address,
        })
    }
}
