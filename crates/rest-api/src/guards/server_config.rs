// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::{entities::server_config, ServerConfig, ServerConfigRepository};

use crate::features::init::ServerConfigNotInitialized;

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for server_config::Model {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match ServerConfigRepository::get(db).await {
            Ok(Some(model)) => Outcome::Success(model),
            Ok(None) => Error::from(ServerConfigNotInitialized).into(),
            Err(err) => Error::from(err).into(),
        }
    }
}

#[axum::async_trait]
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

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let app_config = try_outcome!(request_state!(req, service::AppConfig));
        let model = try_outcome!(server_config::Model::from_request(req).await);
        Outcome::Success(model.with_default_values_from(app_config))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for ServerConfig {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let model = server_config::Model::from_request_parts(parts, state).await?;
        Ok(model.with_default_values_from(&state.app_config))
    }
}
