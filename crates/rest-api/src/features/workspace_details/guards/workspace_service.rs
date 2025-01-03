// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    server_config::ServerConfig,
    workspace::{WorkspaceService, WorkspaceServiceInitError},
};

use crate::{error::prelude::*, guards::prelude::*};

#[axum::async_trait]
impl FromRequestParts<AppState> for WorkspaceService {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let server_config = ServerConfig::from_request_parts(parts, state).await?;

        WorkspaceService::new(
            Arc::new(state.db.clone()),
            Arc::new(state.xmpp_service.clone()),
            Arc::new(state.app_config.clone()),
            &server_config,
            Arc::new(state.secrets_store.clone()),
        )
        .map_err(Error::from)
    }
}

// ERRORS

impl CustomErrorCode for WorkspaceServiceInitError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceXmppAccountNotInitialized => ErrorCode::SERVER_CONFIG_NOT_INITIALIZED,
        }
    }
}
impl_into_error!(WorkspaceServiceInitError);
