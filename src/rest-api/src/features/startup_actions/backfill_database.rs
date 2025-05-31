// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::AppState;

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn backfill_database(
    AppState {
        db,
        app_config,
        network_checker,
        uuid_gen,
        ..
    }: &AppState,
) -> Result<(), String> {
    tracing::debug!("Backfilling database…");

    service::onboarding::backfill(db, app_config, network_checker, uuid_gen).await;

    Ok(())
}
