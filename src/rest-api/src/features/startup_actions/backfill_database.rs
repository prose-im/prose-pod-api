// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::AppState;

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn backfill_database(app_state: &AppState) -> Result<(), String> {
    tracing::debug!("Backfilling database…");

    // Backfill onboarding steps status.
    backfill_onboarding_steps(app_state).await?;

    Ok(())
}

pub async fn backfill_onboarding_steps(
    AppState {
        db,
        app_config,
        network_checker,
        user_repository,
        invitation_repository,
        workspace_service,
        ..
    }: &AppState,
) -> Result<(), String> {
    let ref app_config = app_config.clone();
    service::onboarding::backfill(
        db,
        app_config,
        network_checker,
        invitation_repository,
        user_repository,
        workspace_service,
    )
    .await;
    Ok(())
}
