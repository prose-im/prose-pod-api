// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{future::Future, sync::Arc};

use anyhow::Context as _;
use tracing::trace;

use crate::{
    auth::AuthToken, models::DatabaseRwConnectionPools,
    prose_pod_server_service::ProsePodServerService, server_config, AppConfig, ServerConfig,
};

use super::DynamicServerConfig;

#[derive(Debug, Clone)]
pub struct ServerConfigManager {
    db: DatabaseRwConnectionPools,
    app_config: Arc<AppConfig>,
    server_service: ProsePodServerService,
}

impl ServerConfigManager {
    pub fn new(
        db: DatabaseRwConnectionPools,
        app_config: Arc<AppConfig>,
        server_service: ProsePodServerService,
    ) -> Self {
        Self {
            db,
            app_config,
            server_service,
        }
    }
}

impl ServerConfigManager {
    pub(super) async fn update<F>(
        &self,
        update: impl FnOnce(sea_orm::DatabaseTransaction) -> F,
        auth: &AuthToken,
    ) -> anyhow::Result<ServerConfig>
    where
        F: Future<Output = anyhow::Result<sea_orm::DatabaseTransaction>> + 'static,
    {
        use sea_orm::TransactionTrait as _;

        let txn = (self.db.write.begin().await).context("Failed creating transaction")?;

        let old_config = server_config::get(&txn).await?;
        trace!("Updating server config in database…");
        let txn = update(txn).await?;
        let new_config = server_config::get(&txn).await?;

        if new_config != old_config {
            trace!("Server config has changed, reloading…");
            self.apply(&new_config, Some(auth)).await?;
        } else {
            trace!("Server config hasn’t changed, no need to reload.");
        }

        (txn.commit().await).context("Failed committing transaction")?;

        Ok(ServerConfig::with_default_values(
            &new_config,
            &self.app_config,
        ))
    }

    pub async fn reload(&self, auth: &AuthToken) -> anyhow::Result<ServerConfig> {
        let ref dynamic_server_config = server_config::get(&self.db.read).await?;
        self.apply(dynamic_server_config, Some(auth)).await
    }

    pub async fn apply(
        &self,
        dynamic_server_config: &DynamicServerConfig,
        auth: Option<&AuthToken>,
    ) -> anyhow::Result<ServerConfig> {
        let ref server_service = self.server_service;
        let ref app_config = self.app_config;

        trace!("Saving server config…");
        let server_config = ServerConfig::with_default_values(dynamic_server_config, &app_config);
        server_service
            .save_config(&server_config, &app_config, auth)
            .await?;

        trace!("Reloading XMPP server…");
        server_service.reload(auth).await?;

        Ok(server_config)
    }
}
