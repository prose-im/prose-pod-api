// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::repositories::ServerConfigRepository;
use service::services::{server_ctl::ServerCtl, server_manager::ServerManager};

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest};

/// WARN: Use only in initialization routes! Otherwise use `guards::ServerManager`.
pub struct UnauthenticatedServerManager<'r>(pub(super) ServerManager<'r>);

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedServerManager<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        let server_ctl =
            try_outcome!(req
                .guard::<&State<ServerCtl>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<ServerCtl>` from a request.".to_string(),
                    }
                )));

        let app_config = try_outcome!(req
            .guard::<&State<service::config::Config>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<service::config::Config>` from a request."
                        .to_string(),
                }
            )));

        match ServerConfigRepository::get(db).await {
            Ok(Some(server_config)) => Outcome::Success(Self(ServerManager::new(
                db,
                app_config,
                server_ctl,
                server_config,
            ))),
            Ok(None) => Error::ServerConfigNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}
