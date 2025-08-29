// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::OptionalFromRequestParts;
use service::workspace::WorkspaceService;

use crate::{error::prelude::*, extractors::prelude::*};

impl FromRequestParts<AppState> for WorkspaceService {
    type Rejection = error::Error;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let workspace_jid = state.app_config_frozen().workspace_jid();
        WorkspaceService::new(
            state.xmpp_service.clone(),
            workspace_jid,
            Arc::new(state.base.secrets_store.clone()),
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
