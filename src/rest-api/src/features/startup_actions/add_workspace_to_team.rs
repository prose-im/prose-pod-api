// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::{errors::ServerConfigNotInitialized, ServerConfigRepository};
use tracing::{debug, info, instrument};

use crate::AppState;

/// NOTE: Users need to receive PEP events when the workspace vCard changes for
///   Prose to update the UI automatically. For this, the Workspace needs to be
///   in their rosters. By adding the Workspace to the team, it will automatically
///   be added to everyone’s rosters.
#[instrument(level = "trace", skip_all, err)]
pub async fn add_workspace_to_team(
    AppState {
        db,
        server_ctl,
        app_config,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Adding the Workspace XMPP account to everyone’s rosters…");

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!(
                "Not adding the Workspace XMPP account to everyone’s rosters: {err}",
                err = ServerConfigNotInitialized,
            );
            return Ok(());
        }
        Err(err) => {
            return Err(format!(
                "Could not add the Workspace XMPP account to everyone’s rosters: {err}"
            ));
        }
    };

    let workspace_jid = app_config.workspace_jid(&server_config.domain);

    if let Err(err) = server_ctl.add_team_member(&workspace_jid).await {
        return Err(format!(
            "Could not add the Workspace XMPP account to everyone’s rosters: {err}"
        ));
    }

    Ok(())
}
