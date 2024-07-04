// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::server_config;
use rocket::Request;
use rocket::{outcome::try_outcome, request::Outcome};
use service::Query;

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest};

// TODO: Make it so we can call `server_config.field` directly
// instead of `server_config.model.field`.
#[repr(transparent)]
pub struct ServerConfig(server_config::Model);

impl ServerConfig {
    pub fn model(self) -> server_config::Model {
        self.0
    }
}

impl Deref for ServerConfig {
    type Target = server_config::Model;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match Query::server_config(db).await {
            Ok(Some(server_config)) => Outcome::Success(Self(server_config)),
            Ok(None) => Error::ServerConfigNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}
