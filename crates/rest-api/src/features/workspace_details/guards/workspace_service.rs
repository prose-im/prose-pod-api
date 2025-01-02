// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{
    secrets::SecretsStore,
    server_config::ServerConfig,
    workspace::{WorkspaceService, WorkspaceServiceInitError},
    xmpp::XmppServiceInner,
    AppConfig,
};

use crate::{error::prelude::*, guards::prelude::*};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for WorkspaceService {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let xmpp_service = try_outcome!(request_state!(req, XmppServiceInner));
        let app_config = try_outcome!(request_state!(req, AppConfig));
        let server_config = try_outcome!(ServerConfig::from_request(req).await);
        let secrets_store = try_outcome!(request_state!(req, SecretsStore));

        match WorkspaceService::new(
            Arc::new(db.clone()),
            Arc::new(xmpp_service.clone()),
            Arc::new(app_config.clone()),
            &server_config,
            Arc::new(secrets_store.clone()),
        ) {
            Ok(service) => Outcome::Success(service),
            Err(err) => Error::from(err).into(),
        }
    }
}

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
