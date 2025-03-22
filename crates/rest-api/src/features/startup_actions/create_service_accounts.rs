// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{server_config::ServerConfigRepository, xmpp::ServerManager};
use tracing::{debug, info, instrument};

use crate::{features::init::ServerConfigNotInitialized, AppState};

#[instrument(level = "trace", skip_all, err)]
pub async fn create_service_accounts(
    AppState {
        db,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Creating service accounts…");

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!("Not creating service accounts: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Could not create service accounts: {err}"));
        }
    };

    // Ensure service accounts exist and rotate passwords
    // NOTE: After an update, the Prose Pod API might require more service accounts
    //   than it did when the Prose Pod was initialized. We have to create them before
    //   the Prose Pod API launches.
    if let Err(err) = ServerManager::create_service_accounts(
        &server_config.domain,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
    )
    .await
    {
        return Err(format!("Could not create service accounts: {err}"));
    }

    Ok(())
}
