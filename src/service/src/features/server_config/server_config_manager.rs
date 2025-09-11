// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{future::Future, sync::Arc};

use anyhow::Context as _;
use tracing::trace;

use crate::{sea_orm::DatabaseConnection, server_config, xmpp::ServerCtl, AppConfig, ServerConfig};

use super::DynamicServerConfig;

#[derive(Debug, Clone)]
pub struct ServerConfigManager {
    db: Arc<DatabaseConnection>,
    app_config: Arc<AppConfig>,
    server_ctl: Arc<ServerCtl>,
}

impl ServerConfigManager {
    pub fn new(
        db: Arc<DatabaseConnection>,
        app_config: Arc<AppConfig>,
        server_ctl: Arc<ServerCtl>,
    ) -> Self {
        Self {
            db,
            app_config,
            server_ctl,
        }
    }
}

impl ServerConfigManager {
    pub(super) async fn update<F>(
        &self,
        update: impl FnOnce(sea_orm::DatabaseTransaction) -> F,
    ) -> anyhow::Result<ServerConfig>
    where
        F: Future<Output = anyhow::Result<sea_orm::DatabaseTransaction>> + 'static,
    {
        use sea_orm::TransactionTrait as _;

        let txn = (self.db.as_ref().begin().await).context("Failed creating transaction")?;

        let old_config = server_config::get(&txn).await?;
        trace!("Updating server config in database…");
        let txn = update(txn).await?;
        let new_config = server_config::get(&txn).await?;

        if new_config != old_config {
            trace!("Server config has changed, reloading…");
            self.apply(&new_config).await?;
        } else {
            trace!("Server config hasn’t changed, no need to reload.");
        }

        (txn.commit().await).context("Failed committing transaction")?;

        Ok(ServerConfig::with_default_values(
            &new_config,
            &self.app_config,
        ))
    }

    pub async fn reload(&self) -> anyhow::Result<ServerConfig> {
        let db = self.db.as_ref();
        let ref dynamic_server_config = server_config::get(db).await?;
        self.apply(dynamic_server_config).await
    }

    pub async fn apply(
        &self,
        dynamic_server_config: &DynamicServerConfig,
    ) -> anyhow::Result<ServerConfig> {
        let server_ctl = self.server_ctl.as_ref();
        let ref app_config = self.app_config;

        trace!("Saving server config…");
        let server_config = ServerConfig::with_default_values(dynamic_server_config, &app_config);
        server_ctl.save_config(&server_config, &app_config).await?;

        trace!("Reloading XMPP server…");
        server_ctl.reload().await?;

        Ok(server_config)
    }
}
