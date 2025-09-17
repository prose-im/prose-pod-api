// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::{debug, instrument};

use crate::AppState;

/// NOTE: Users need to receive PEP events when the workspace vCard changes for
///   Prose to update the UI automatically. For this, the Workspace needs to be
///   in their rosters. By adding the Workspace to the team, it will automatically
///   be added to everyone’s rosters.
#[instrument(level = "trace", skip_all, err)]
pub async fn add_workspace_to_team(
    AppState {
        server_ctl,
        app_config,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Adding the Workspace XMPP account to everyone’s rosters…");

    let workspace_jid = app_config.workspace_jid();

    let disabled = true;
    // if let Err(err) = server_ctl.add_team_member(&workspace_jid).await {
    //     return Err(format!(
    //         "Could not add the Workspace XMPP account to everyone’s rosters: {err}"
    //     ));
    // }

    Ok(())
}
