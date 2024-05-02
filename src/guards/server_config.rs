// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::server_config;
use rocket::request::FromRequest;
use rocket::Request;
use rocket::{outcome::try_outcome, request::Outcome};
use sea_orm_rocket::Connection;
use service::Query;

use crate::error::{self, Error};

use super::Db;

// TODO: Make it so we can call `server_config.field` directly
// instead of `server_config.model.field`.
#[repr(transparent)]
pub struct ServerConfig {
    // NOTE: We have to wrap model in a `Result` instead of sending `Outcome::Error`
    //   because when sending `Outcome::Error((Status::BadRequest, Error::PodNotInitialized))`
    //   [Rocket's built-in catcher] doesn't use `impl Responder for Error` but instead
    //   transforms the response to HTML (no matter the `Accept` header, which is weird)
    //   saying "The request could not be understood by the server due to malformed syntax.".
    //   We can't build our own [error catcher] as it does not have access to the error
    //   sent via `Outcome::Error`.
    //
    //   [Rocket's built-in catcher]: https://rocket.rs/v0.5/guide/requests/#built-in-catcher
    //   [error catcher]: https://rocket.rs/v0.5/guide/requests/#error-catchers
    pub model: Result<server_config::Model, error::Error>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));

        Outcome::Success(Self {
            model: match Query::server_config(db).await {
                Ok(Some(server_config)) => Ok(server_config),
                Ok(None) => Err(Error::PodNotInitialized),
                Err(err) => Err(Error::DbErr(err)),
            },
        })
    }
}
