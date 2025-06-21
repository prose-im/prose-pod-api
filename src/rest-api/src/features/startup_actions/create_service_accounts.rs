// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::xmpp::server_manager;
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn create_service_accounts(
    AppState {
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Creating service accounts…");

    let ref app_config = app_config.read().unwrap().clone();

    // Ensure service accounts exist and rotate passwords
    // NOTE: After an update, the Prose Pod API might require more service accounts
    //   than it did when the Prose Pod was initialized. We have to create them before
    //   the Prose Pod API launches.
    if let Err(err) =
        server_manager::create_service_accounts(server_ctl, app_config, auth_service, secrets_store)
            .await
    {
        return Err(format!("Could not create service accounts: {err}"));
    }

    Ok(())
}
