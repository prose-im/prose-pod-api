// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::features::server_config::{
    entities::server_config, ServerConfig, ServerConfigRepository,
};

use crate::features::init::ServerConfigNotInitialized;

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for server_config::Model {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match ServerConfigRepository::get(db).await {
            Ok(Some(model)) => Outcome::Success(model),
            Ok(None) => Error::from(ServerConfigNotInitialized).into(),
            Err(err) => Error::from(err).into(),
        }
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let app_config = try_outcome!(request_state!(req, service::AppConfig));
        let model = try_outcome!(server_config::Model::from_request(req).await);
        Outcome::Success(model.with_default_values_from(app_config))
    }
}
