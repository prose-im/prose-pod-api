// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::FromRef;
use service::server_config::{self, ServerConfigManager};
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn init_server_config(app_state @ AppState { db, .. }: &AppState) -> Result<(), String> {
    debug!("Initializing the XMPP server configuration…");

    let server_config = (server_config::get(&db.read).await)
        .map_err(|err| format!("Could not initialize the XMPP server configuration: {err}"))?;

    // Apply the server configuration stored in the database
    let server_config_manager = ServerConfigManager::from_ref(app_state);
    if let Err(err) = server_config_manager.apply(&server_config).await {
        return Err(format!(
            "Could not initialize the XMPP server configuration: {err}"
        ));
    }

    Ok(())
}
