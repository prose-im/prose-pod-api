// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::MemberRepository;
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn validate_app_config_changes(app_state: &AppState) -> Result<(), String> {
    debug!("Validating static config changes…");

    ensure_server_domain_not_changed(app_state).await?;

    Ok(())
}

async fn ensure_server_domain_not_changed(app_state: &AppState) -> Result<(), String> {
    let (_, members) = (MemberRepository::get_page(&app_state.db, 1, 1, None).await)
        .map_err(|err| format!("Could not ensure the server domain hasn’t been modified: {err}"))?;

    let Some(member) = members.first() else {
        debug!(
            "Could not ensure the server domain hasn’t been modified: First account not initialized.",
        );
        return Ok(());
    };

    let ref app_config = app_state.app_config_frozen();

    let old_domain = member.jid().domain().to_string();
    let new_domain = app_config.server_domain().to_string();
    if new_domain != old_domain {
        debug!("Server domain changed from {old_domain} to {new_domain}.");
        return Err("You can’t change the server domain after the first startup.".to_owned());
    }

    Ok(())
}
