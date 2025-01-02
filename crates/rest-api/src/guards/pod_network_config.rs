// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    network_checks::PodNetworkConfig,
    pod_config::{PodAddress, PodConfigRepository},
    server_config::ServerConfig,
};

use crate::features::init::PodAddressNotInitialized;

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for PodNetworkConfig {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        try_outcome!(check_caller_is_admin(req, None).await);

        let db = try_outcome!(database_connection(req).await);
        let server_domain = try_outcome!(ServerConfig::from_request(req).await).domain;

        let pod_config = match PodConfigRepository::get(db).await {
            Ok(Some(model)) => model,
            Ok(None) => return Error::from(PodAddressNotInitialized).into(),
            Err(err) => return Error::from(err).into(),
        };
        let pod_address = match PodAddress::try_from(pod_config) {
            Ok(pod_address) => pod_address,
            Err(err) => return Error::from(err).into(),
        };

        Outcome::Success(PodNetworkConfig {
            server_domain,
            pod_address,
        })
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for PodNetworkConfig {
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

        Ok(PodNetworkConfig {
            server_domain,
            pod_address,
        })
    }
}
