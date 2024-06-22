// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{RwLock, RwLockWriteGuard};

use entity::model::{DateLike, Duration, PossiblyInfinite};
use entity::server_config;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use sea_orm_rocket::Connection;
use service::config::Config as AppConfig;
use service::sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set, TransactionTrait as _};
use service::{Mutation, Query, ServerCtl};

use crate::error::{self, Error};

use super::{Db, LazyFromRequest};

/// WARN: Use only in initialization routes! Otherwise use `ServerManager`.
pub struct UnauthenticatedServerManager<'r> {
    db: &'r DatabaseConnection,
    app_config: &'r AppConfig,
    server_ctl: &'r ServerCtl,
    server_config: RwLock<server_config::Model>,
}

impl<'r> UnauthenticatedServerManager<'r> {
    pub fn new(
        db: &'r DatabaseConnection,
        app_config: &'r AppConfig,
        server_ctl: &'r ServerCtl,
        server_config: server_config::Model,
    ) -> Self {
        Self {
            db,
            app_config,
            server_ctl,
            server_config: RwLock::new(server_config),
        }
    }

    fn server_config_mut(&self) -> RwLockWriteGuard<server_config::Model> {
        self.server_config
            .write()
            .expect("`server_config::Model` lock poisonned")
    }

    pub fn server_config(&self) -> server_config::Model {
        self.server_config_mut().to_owned()
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedServerManager<'r> {
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
            Ok(Some(server_config)) => Outcome::Success(UnauthenticatedServerManager::new(
                db,
                app_config,
                server_ctl,
                server_config,
            )),
            Ok(None) => Error::ServerConfigNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}

impl<'r> UnauthenticatedServerManager<'r> {
    async fn update<U>(&self, update: U) -> Result<server_config::Model, Error>
    where
        U: FnOnce(&mut server_config::ActiveModel) -> (),
    {
        let old_server_config = self.server_config_mut().clone();

        let mut active: server_config::ActiveModel = old_server_config.clone().into();
        update(&mut active);
        trace!("Updating config in database…");
        let new_server_config = active.update(self.db).await?;
        *self.server_config_mut() = new_server_config.clone();

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
        self.reload(&self.server_config_mut())
    }

    /// Reload the XMPP server using the server configuration passed as an argument.
    fn reload(&self, server_config: &server_config::Model) -> Result<(), Error> {
        let server_ctl = self.server_ctl.read().expect("ServerCtl lock poisonned");

        // Save new server config
        trace!("Saving server config…");
        server_ctl.save_config(&server_config, self.app_config)?;
        // Reload server config
        trace!("Reloading XMPP server…");
        server_ctl.reload()?;

        Ok(())
    }
}

impl<'r> UnauthenticatedServerManager<'r> {
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

    pub async fn init_server_config(
        db: &DatabaseConnection,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        server_config: server_config::ActiveModel,
    ) -> Result<server_config::Model, Error> {
        let None = Query::server_config(db).await? else {
            return Err(Error::ServerConfigAlreadyInitialized);
        };

        let txn = db.begin().await?;

        // Initialize the server config in a transaction,
        // to rollback if subsequent operations fail.
        let server_config = Mutation::create_server_config(&txn, server_config).await?;

        // NOTE: We can't rollback changes made to the XMPP server so let's do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        {
            let server_ctl = server_ctl.implem.read().expect("Serverctl lock poisonned");
            server_ctl.save_config(&server_config, app_config)?;
            server_ctl.reload()?;
        }

        // Commit the transaction only if the admin user was
        // successfully created, to prevent inconsistent states.
        txn.commit().await?;

        Ok(server_config)
    }

    pub async fn set_domain(&self, domain: &str) -> Result<server_config::Model, Error> {
        trace!("Setting XMPP server domain to {domain}…");
        self.update(|active| {
            active.domain = Set(domain.to_owned());
        })
        .await
    }

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
