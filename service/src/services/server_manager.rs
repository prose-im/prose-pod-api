// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{RwLock, RwLockWriteGuard};

use entity::server_config;
use sea_orm::DbErr;
use tracing::trace;

use crate::config::Config as AppConfig;
use crate::model::{DateLike, Duration, PossiblyInfinite, ServerConfig};
use crate::repositories::{ServerConfigCreateForm, ServerConfigRepository};
use crate::sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set, TransactionTrait as _};
use crate::services::server_ctl::ServerCtl;

use super::server_ctl;

pub struct ServerManager<'r> {
    db: &'r DatabaseConnection,
    app_config: &'r AppConfig,
    server_ctl: &'r ServerCtl,
    server_config: RwLock<ServerConfig>,
}

impl<'r> ServerManager<'r> {
    pub fn new(
        db: &'r DatabaseConnection,
        app_config: &'r AppConfig,
        server_ctl: &'r ServerCtl,
        server_config: ServerConfig,
    ) -> Self {
        Self {
            db,
            app_config,
            server_ctl,
            server_config: RwLock::new(server_config),
        }
    }

    fn server_config_mut(&self) -> RwLockWriteGuard<ServerConfig> {
        self.server_config
            .write()
            .expect("`ServerConfig` lock poisonned")
    }

    pub fn server_config(&self) -> ServerConfig {
        self.server_config_mut().to_owned()
    }
}

impl<'r> ServerManager<'r> {
    async fn update<U>(&self, update: U) -> Result<ServerConfig, Error>
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
    pub fn reload_current(&self) -> Result<(), Error> {
        self.reload(&self.server_config_mut())
    }

    /// Reload the XMPP server using the server configuration passed as an argument.
    fn reload(&self, server_config: &ServerConfig) -> Result<(), Error> {
        let server_ctl = self.server_ctl;

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

    pub async fn init_server_config(
        db: &DatabaseConnection,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        server_config: impl Into<ServerConfigCreateForm>,
    ) -> Result<ServerConfig, Error> {
        let None = ServerConfigRepository::get(db).await? else {
            return Err(Error::ServerConfigAlreadyInitialized);
        };

        let txn = db.begin().await?;

        // Initialize the server config in a transaction,
        // to rollback if subsequent operations fail.
        let server_config = ServerConfigRepository::create(&txn, server_config).await?;

        // NOTE: We can't rollback changes made to the XMPP server so let's do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        {
            server_ctl.save_config(&server_config, app_config)?;
            server_ctl.reload()?;
        }

        // Commit the transaction only if the admin user was
        // successfully created, to prevent inconsistent states.
        txn.commit().await?;

        Ok(server_config)
    }

    pub async fn set_domain(&self, domain: &str) -> Result<ServerConfig, Error> {
        trace!("Setting XMPP server domain to {domain}…");
        self.update(|active| {
            active.domain = Set(domain.to_owned());
        })
        .await
    }

    pub async fn set_message_archiving(&self, new_state: bool) -> Result<ServerConfig, Error> {
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
    ) -> Result<ServerConfig, Error> {
        trace!("Setting message archive retention to {new_state}…");
        self.update(|active| {
            active.message_archive_retention = Set(new_state);
        })
        .await
    }

    pub async fn set_file_uploading(&self, new_state: bool) -> Result<ServerConfig, Error> {
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
    ) -> Result<ServerConfig, Error> {
        trace!("Setting file retention to {new_state}…");
        self.update(|active| {
            active.file_storage_retention = Set(new_state);
        })
        .await
    }
}

pub type Error = ServerManagerError;

#[derive(Debug, thiserror::Error)]
pub enum ServerManagerError {
    #[error("XMPP server already initialized.")]
    ServerConfigAlreadyInitialized,
    #[error("`ServerCtl` error: {0}")]
    ServerCtl(#[from] server_ctl::Error),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
