// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::Request;
use rocket::{outcome::try_outcome, request::Outcome};
use service::repositories::ServerConfigRepository;

use crate::error::{self, Error};

use super::{util::database_connection, LazyFromRequest};

type ServerConfigModel = service::repositories::ServerConfig;

// TODO: Make it so we can call `server_config.field` directly
// instead of `server_config.model.field`.
#[repr(transparent)]
pub struct ServerConfig(ServerConfigModel);

impl ServerConfig {
    pub fn model(self) -> ServerConfigModel {
        self.0
    }
}

impl Deref for ServerConfig {
    type Target = ServerConfigModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match ServerConfigRepository::get(db).await {
            Ok(Some(server_config)) => Outcome::Success(Self(server_config)),
            Ok(None) => Error::ServerConfigNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}
