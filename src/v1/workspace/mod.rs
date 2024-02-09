// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(super) mod openapi_extensions;
pub mod reactions;
pub mod routes;

use entity::settings;
pub use routes::*;

use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Route};
use sea_orm_rocket::Connection;
use service::Query;

use crate::pool::Db;

use super::error::{self, Error};

pub(super) fn routes() -> Vec<Route> {
    vec![
        reactions::routes(),
        self::_routes(),
    ]
    .concat()
}

fn _routes() -> Vec<Route> {
    routes![
        get_workspace_name,
        set_workspace_name,
        get_workspace_icon,
        set_workspace_icon_string,
        set_workspace_icon_file,
        get_workspace_details_card,
        set_workspace_details_card,
        get_workspace_accent_color,
        set_workspace_accent_color,
    ]
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
