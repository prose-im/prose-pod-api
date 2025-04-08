// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod refresh_service_accounts_tokens;

use futures::future::join_all;
use sea_orm::DatabaseConnection;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::AppConfig;

use super::{auth::AuthService, secrets::SecretsStore};

#[derive(Debug, Clone)]
pub struct CronContext {
    pub cancellation_token: CancellationToken,
    pub app_config: AppConfig,
    pub db: DatabaseConnection,
    pub secrets_store: SecretsStore,
    pub auth_service: AuthService,
}

pub fn start_cron_tasks(ctx: CronContext) {
    info!("Staring periodic tasks…");

    macro_rules! spawn {
        ($task:ident) => {{
            let ctx = $task::Context::from(ctx.clone());
            tokio::spawn(async move { $task::run(ctx).await })
        }};
    }

    tokio::spawn(async move {
        tokio::select! {
            _ = join_all(vec![spawn!(refresh_service_accounts_tokens)]) => {},
            _ = ctx.cancellation_token.cancelled() => {},
        }
    });
}
