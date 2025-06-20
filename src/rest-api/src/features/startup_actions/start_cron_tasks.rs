// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::cron::CronContext;
use tracing::instrument;

use crate::{AppState, MinimalAppState};

#[instrument(level = "trace", skip_all, err)]
pub async fn start_cron_tasks(
    AppState {
        base: MinimalAppState {
            lifecycle_manager, ..
        },
        db,
        app_config,
        secrets_store,
        auth_service,
        ..
    }: &AppState,
) -> Result<(), String> {
    let ctx = CronContext {
        cancellation_token: lifecycle_manager.child_cancellation_token(),
        app_config: app_config.clone(),
        db: db.to_owned(),
        secrets_store: secrets_store.to_owned(),
        auth_service: auth_service.to_owned(),
    };
    service::cron::start_cron_tasks(ctx);
    Ok(())
}
