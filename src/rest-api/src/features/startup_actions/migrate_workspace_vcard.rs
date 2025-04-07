// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{server_config::ServerConfigRepository, workspace::WorkspaceService};
use tracing::{debug, info, instrument};

use crate::{
    features::init::{ServerConfigNotInitialized, WorkspaceNotInitialized},
    AppState,
};

#[instrument(level = "trace", skip_all, err)]
pub async fn migrate_workspace_vcard(
    AppState {
        db,
        app_config,
        xmpp_service,
        secrets_store,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Migrating the Workspace vCard…");

    #[cfg(debug_assertions)]
    if (app_config.debug_only.skip_startup_actions).contains("migrate_workspace_vcard") {
        info!("Not migrating the Workspace vCard: Step marked to skip in the app configuration.");
        return Ok(());
    }

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!("Not migrating the Workspace vCard: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Could not migrate the Workspace vCard: {err}"));
        }
    }
    .with_default_values_from(app_config);

    let workspace_service = WorkspaceService::new(
        Arc::new(xmpp_service.clone()),
        Arc::new(app_config.clone()),
        &server_config,
        Arc::new(secrets_store.clone()),
    )
    .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?;

    if !workspace_service
        .is_workspace_initialized()
        .await
        .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?
    {
        info!("Not migrating the Workspace vCard: {WorkspaceNotInitialized}");
        return Ok(());
    }

    workspace_service
        .migrate_workspace_vcard()
        .await
        .map_err(|err| format!("Could not migrate the Workspace vCard: {err}"))?;

    Ok(())
}
