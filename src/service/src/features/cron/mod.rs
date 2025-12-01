// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

// use futures::future::join_all;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{models::DatabaseRwConnectionPools, AppConfig};

use super::auth::AuthService;

#[derive(Debug, Clone)]
pub struct CronContext {
    pub cancellation_token: CancellationToken,
    pub app_config: Arc<AppConfig>,
    pub db: DatabaseRwConnectionPools,
    pub auth_service: AuthService,
}

pub fn start_cron_tasks(ctx: CronContext) {
    info!("Starting periodic tasks…");

    // macro_rules! spawn {
    //     ($task:ident) => {{
    //         let ctx = $task::Context::from(ctx.clone());
    //         tokio::spawn(async move { $task::run(ctx).await }.in_current_span())
    //     }};
    // }

    tokio::spawn(
        async move {
            tokio::select! {
                // _ = join_all(vec![
                //     spawn!(todo),
                // ]) => {},
                _ = ctx.cancellation_token.cancelled() => {},
            }
        },
        // FIXME: For some reason, this breaks behavior tests.
        // .in_current_span(),
    );
}
