// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Mutex, MutexGuard};

use entity::model::{DateLike, Duration, PossiblyInfinite};
use entity::server_config;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use sea_orm_rocket::Connection;
use service::config::Config;
use service::sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set};
use service::{Query, ServerCtl};

use crate::error::{self, Error};

use super::{Db, FromRequest, JID as JIDGuard};

pub struct ServerManager<'r> {
    db: &'r DatabaseConnection,
    app_config: &'r Config,
    server_ctl: &'r ServerCtl,
    server_config: Mutex<server_config::Model>,
}

impl<'r> ServerManager<'r> {
    pub(crate) fn new(
        db: &'r DatabaseConnection,
        app_config: &'r Config,
        server_ctl: &'r ServerCtl,
        server_config: server_config::Model,
    ) -> Self {
        Self {
            db,
            app_config,
            server_ctl,
            server_config: Mutex::new(server_config),
        }
    }

    fn server_config(&self) -> MutexGuard<server_config::Model> {
        self.server_config
            .lock()
            .expect("`server_config::Model` lock poisonned")
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

        let jid = try_outcome!(JIDGuard::from_request(req).await);
        match Query::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                debug!("<{}> is not an admin", jid.to_string());
                return Error::Unauthorized.into();
            }
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

        match Query::server_config(db).await {
            Ok(Some(server_config)) => Outcome::Success(ServerManager::new(
                db,
                app_config,
                server_ctl,
                server_config,
            )),
            Ok(None) => Error::PodNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}

impl<'r> ServerManager<'r> {
    async fn update<U>(&self, update: U) -> Result<server_config::Model, Error>
    where
        U: FnOnce(&mut server_config::ActiveModel) -> (),
    {
        let old_server_config = self.server_config().clone();

        let mut active: server_config::ActiveModel = old_server_config.clone().into();
        update(&mut active);
        trace!("Updating config in database…");
        let new_server_config = active.update(self.db).await?;
        *self.server_config() = new_server_config.clone();

        if new_server_config != old_server_config {
            trace!("Server config has changed, reloading…");
            self.reload(&new_server_config)?;
        } else {
            trace!("Server config hasn't changed, no need to reload.");
        }

        Ok(new_server_config)
    }

    /// Reload the XMPP server using the server configuration stored in `self`.
    pub(crate) fn reload_current(&self) -> Result<(), Error> {
        self.reload(&self.server_config())
    }

    /// Reload the XMPP server using the server configuration passed as an argument.
    fn reload(&self, server_config: &server_config::Model) -> Result<(), Error> {
        let server_ctl = self.server_ctl.lock().expect("Serverctl lock poisonned");

        // Save new server config
        trace!("Saving server config…");
        server_ctl.save_config(&server_config, self.app_config)?;
        // Reload server config
        trace!("Reloading XMPP server…");
        server_ctl.reload()?;

        Ok(())
    }
}

impl<'r> ServerManager<'r> {
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
        trace!(
            "Turning {} message archiving…",
            if new_state { "on" } else { "off" }
        );
        self.update(|active| {
            active.message_archive_enabled = Set(new_state);
        })
        .await
    }
    pub async fn set_message_archive_retention(
        &self,
        new_state: PossiblyInfinite<Duration<DateLike>>,
    ) -> Result<server_config::Model, Error> {
        trace!("Setting message archive retention to {new_state}…");
        self.update(|active| {
            active.message_archive_retention = Set(new_state);
        })
        .await
    }

    pub async fn set_file_uploading(&self, new_state: bool) -> Result<server_config::Model, Error> {
        trace!(
            "Turning {} file uploading…",
            if new_state { "on" } else { "off" }
        );
        self.update(|active| {
            active.file_upload_allowed = Set(new_state);
        })
        .await
    }
    pub async fn set_file_retention(
        &self,
        new_state: PossiblyInfinite<Duration<DateLike>>,
    ) -> Result<server_config::Model, Error> {
        trace!("Setting file retention to {new_state}…");
        self.update(|active| {
            active.file_storage_retention = Set(new_state);
        })
        .await
    }
}
