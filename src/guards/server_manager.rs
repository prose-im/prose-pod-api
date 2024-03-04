// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::model::{DateLike, Duration, PossiblyInfinite};
use entity::server_config;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use sea_orm_rocket::Connection;
use service::sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set};
use service::Query;

use crate::error::{self, Error};
use crate::server_ctl::ServerCtl;

use super::{Db, JID as JIDGuard};

pub struct ServerManager<'r> {
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
    pub inner: Result<ServerManagerInner<'r>, Error>,
}

impl<'r> Deref for ServerManager<'r> {
    type Target = Result<ServerManagerInner<'r>, Error>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ServerManager<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));

        let jid = try_outcome!(req.guard::<JIDGuard>().await);
        match Query::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => return Error::Unauthorized.into(),
            Err(e) => return Error::DbErr(e).into(),
        }

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
                Ok(Some(server_config)) => Ok(ServerManagerInner {
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

pub struct ServerManagerInner<'r> {
    db: &'r DatabaseConnection,
    server_ctl: &'r State<ServerCtl>,
    server_config: server_config::Model,
}

impl<'r> ServerManagerInner<'r> {
    async fn update<U>(&self, update: U) -> Result<server_config::Model, Error>
    where
        U: FnOnce(&mut server_config::ActiveModel) -> (),
    {
        let config_before = &self.server_config;
        let mut active: server_config::ActiveModel = self.server_config.clone().into();
        update(&mut active);
        let server_config = active.update(self.db).await.map_err(Error::DbErr)?;

        if server_config != *config_before {
            self.server_ctl
                .lock()
                .expect("Serverctl lock poisonned")
                .reload()
                .map_err(Error::ServerCtlErr)?;
        }

        Ok(server_config)
    }
}

impl<'r> ServerManagerInner<'r> {
    // TODO: Use or delete the following comments

    // pub fn add_admin(&self, jid: JID) {
    //     todo!()
    // }
    // pub fn remove_admin(&self, jid: &JID) {
    //     todo!()
    // }

    // pub fn set_rate_limit(&self, conn_type: ConnectionType, value: DataRate) {
    //     todo!()
    // }
    // pub fn set_burst_limit(&self, conn_type: ConnectionType, value: Duration<TimeLike>) {
    //     todo!()
    // }
    // /// Sets the time that an over-limit session is suspended for
    // /// (`limits_resolution` in Prosody).
    // ///
    // /// See <https://prosody.im/doc/modules/mod_limits> for Prosody
    // /// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
    // pub fn set_timeout(&self, value: Duration<TimeLike>) {
    //     todo!()
    // }

    pub async fn set_message_archiving(
        &self,
        new_state: bool,
    ) -> Result<server_config::Model, Error> {
        self.update(|active| {
            active.message_archive_enabled = Set(new_state);
        })
        .await
    }
    pub async fn set_message_archive_retention(
        &self,
        new_state: PossiblyInfinite<Duration<DateLike>>,
    ) -> Result<server_config::Model, Error> {
        self.update(|active| {
            active.message_archive_retention = Set(new_state);
        })
        .await
    }

    pub async fn set_file_uploading(&self, new_state: bool) -> Result<server_config::Model, Error> {
        self.update(|active| {
            active.file_upload_allowed = Set(new_state);
        })
        .await
    }
    pub async fn set_file_retention(
        &self,
        new_state: PossiblyInfinite<Duration<DateLike>>,
    ) -> Result<server_config::Model, Error> {
        self.update(|active| {
            active.file_storage_retention = Set(new_state);
        })
        .await
    }
}
