// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::{entities::server_config, ServerConfigRepository};

use crate::features::init::ServerConfigNotInitialized;

use super::prelude::*;

impl FromRequestParts<AppState> for server_config::Model {
    type Rejection = error::Error;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match ServerConfigRepository::get(&state.db).await? {
            Some(model) => Ok(model),
            None => Err(Error::from(ServerConfigNotInitialized)),
        }
    }
}

impl FromRequestParts<AppState> for service::server_config::ServerConfig {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let model = server_config::Model::from_request_parts(parts, state).await?;
        Ok(model.with_default_values_from(&state.app_config))
    }
}
