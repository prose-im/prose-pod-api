// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{server_config::ServerConfigRepository, xmpp::ServerManager};
use tracing::{debug, info, instrument};

use crate::{features::init::errors::ServerConfigNotInitialized, AppState};

#[instrument(level = "trace", skip_all, err)]
pub async fn init_server_config(
    AppState {
        db,
        server_ctl,
        app_config,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Initializing the XMPP server configuration…");

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!("Not initializing the XMPP server configuration: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!(
                "Could not initialize the XMPP server configuration: {err}"
            ));
        }
    };

    // Apply the server configuration stored in the database
    let server_manager = ServerManager::new(
        Arc::new(db.clone()),
        Arc::new(app_config.clone()),
        Arc::new(server_ctl.clone()),
        server_config.clone(),
    );
    if let Err(err) = server_manager.reload_current().await {
        return Err(format!(
            "Could not initialize the XMPP server configuration: {err}"
        ));
    }

    Ok(())
}
