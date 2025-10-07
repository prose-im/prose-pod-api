// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::onboarding;
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn validate_app_config_changes(app_state: &AppState) -> Result<(), String> {
    debug!("Validating static config changes…");

    ensure_server_domain_not_changed(app_state).await?;

    Ok(())
}

async fn ensure_server_domain_not_changed(
    AppState { db, app_config, .. }: &AppState,
) -> Result<(), String> {
    let chosen_domain = (onboarding::chosen_server_domain::get_opt(&db.read).await)
        .map_err(|err| format!("Could not ensure the server domain hasn’t been modified: {err}"))?;

    let Some(old_domain) = chosen_domain else {
        debug!(
            "Could not ensure the server domain hasn’t been modified: First account not initialized.",
        );
        return Ok(());
    };

    let new_domain = app_config.server_domain().to_string();
    if new_domain != old_domain {
        debug!("Server domain changed from {old_domain} to {new_domain}.");
        return Err("You can’t change the server domain after the first startup.".to_owned());
    }

    Ok(())
}
