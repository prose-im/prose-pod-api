// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::model::JID;
use entity::server_config;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use sea_orm_rocket::Connection;
use service::sea_orm::DatabaseConnection;
use service::{Query, ServerCtl};

use crate::error::{self, Error};

use super::Db;

pub struct UserFactory<'r> {
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
    pub inner: Result<UserFactoryInner<'r>, Error>,
}

impl<'r> Deref for UserFactory<'r> {
    type Target = Result<UserFactoryInner<'r>, Error>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserFactory<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));

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
        Outcome::Success(Self {
            inner: match Query::server_config(db).await {
                Ok(Some(server_config)) => Ok(UserFactoryInner {
                    db,
                    server_ctl,
                    server_config,
                }),
                Ok(None) => Err(Error::PodNotInitialized),
                Err(err) => Err(Error::DbErr(err)),
            },
        })
    }
}

pub struct UserFactoryInner<'r> {
    db: &'r DatabaseConnection,
    server_ctl: &'r State<ServerCtl>,
    server_config: server_config::Model,
}

impl<'r> UserFactoryInner<'r> {
    pub async fn create_user(
        &self,
        jid: &JID,
        password: &str,
        nickname: &str,
    ) -> Result<(), Error> {
        self.server_ctl
            .lock()
            .expect("Serverctl lock poisonned")
            .add_user(jid, password)
            .map_err(Error::ServerCtlErr)?;

        // FIXME: Set nickname

        Ok(())
    }
}
