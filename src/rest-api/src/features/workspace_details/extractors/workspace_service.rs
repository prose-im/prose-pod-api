// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::OptionalFromRequestParts;
use service::{workspace::WorkspaceService, xmpp::XmppServiceContext};

use crate::{
    error::{prelude::*, InternalServerError},
    extractors::prelude::*,
};

impl FromRequestParts<AppState> for WorkspaceService {
    type Rejection = error::Error;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let workspace_jid = state.app_config.workspace_jid();
        let workspace_token = state
            .base
            .secrets_store
            .get_service_account_prosody_token(&workspace_jid);

        let Some(workspace_token) = workspace_token else {
            return Err(Error::from(InternalServerError(
                "Auth token for Workspace account not found.".to_owned(),
            )));
        };

        Ok(WorkspaceService {
            xmpp_service: state.xmpp_service.clone(),
            ctx: XmppServiceContext {
                bare_jid: workspace_jid,
                auth_token: workspace_token,
            },
        })
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
