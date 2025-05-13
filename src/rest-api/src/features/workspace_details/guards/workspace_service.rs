// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::OptionalFromRequestParts;
use service::{server_config::ServerConfig, workspace::WorkspaceService};

use crate::{error::prelude::*, guards::prelude::*};

impl FromRequestParts<AppState> for WorkspaceService {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_config = ServerConfig::from_request_parts(parts, state).await?;

        WorkspaceService::new(
            Arc::new(state.xmpp_service.clone()),
            Arc::new(state.app_config.clone()),
            &server_config,
            Arc::new(state.secrets_store.clone()),
        )
        .map_err(Error::from)
    }
}

impl OptionalFromRequestParts<AppState> for WorkspaceService {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <WorkspaceService as FromRequestParts<AppState>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}
