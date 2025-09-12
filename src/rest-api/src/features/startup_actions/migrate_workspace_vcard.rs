// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::workspace::{errors::WorkspaceNotInitialized, WorkspaceService};
use tracing::{debug, info, instrument};

use crate::{AppState, MinimalAppState};

#[instrument(level = "trace", skip_all, err)]
pub async fn migrate_workspace_vcard(
    AppState {
        base: MinimalAppState { secrets_store, .. },
        app_config,
        xmpp_service,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Migrating the Workspace vCard…");

    let workspace_jid = app_config.workspace_jid();
    let workspace_service = WorkspaceService::new(
        xmpp_service.clone(),
        workspace_jid,
        Arc::new(secrets_store.clone()),
    )
    .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?;

    if !(workspace_service.is_workspace_initialized().await)
        .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?
    {
        info!(
            "Not migrating the Workspace vCard: {err}",
            err = WorkspaceNotInitialized::NoReason,
        );
        return Ok(());
    }

    (workspace_service.migrate_workspace_vcard().await)
        .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?;

    Ok(())
}
