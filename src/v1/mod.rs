// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod error;
mod members;
pub mod routes;
pub mod server;
pub mod workspace;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
pub use routes::*;

use entity::settings;
use rocket::Route;
use rocket::{routes, Request};
use sea_orm_rocket::Connection;
use service::Query;

use crate::pool::Db;

use self::error::Error;

pub(super) fn routes() -> Vec<Route> {
    vec![
        routes![openapi, init, login],
        members::routes(),
        server::routes(),
        workspace::routes(),
    ]
    .concat()
}

// TODO: Make it so we can call `settings.field` directly
// instead of `settings.model.field`.
#[repr(transparent)]
pub struct Settings {
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
    pub model: Result<settings::Model, error::Error>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Settings {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            });
        let db = match db {
            Outcome::Success(db) => db,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(e) => return Outcome::Forward(e),
        };

        Outcome::Success(Self {
            model: match Query::settings(db).await {
                Ok(Some(settings)) => Ok(settings),
                Ok(None) => Err(Error::PodNotInitialized),
                Err(err) => Err(Error::DbErr(err)),
            },
        })
    }
}

pub struct Admin {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(auth) = req.headers().get_one("Authorization") else {
            return Outcome::Error((Status::Unauthorized, Error::Unauthorized));
        };

        match auth {
            "ok" => Outcome::Success(Self {}),
            _ => Outcome::Error((Status::Unauthorized, Error::Unauthorized)),
        }
    }
}
