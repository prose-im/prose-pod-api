// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::xmpp::ServerManager;
use tracing::debug;

use crate::AppState;

pub async fn rotate_api_xmpp_password(
    AppState {
        server_ctl,
        app_config,
        secrets_store,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Rotating Prose Pod API's XMPP password…");

    if let Err(err) =
        ServerManager::rotate_api_xmpp_password(server_ctl, app_config, secrets_store).await
    {
        return Err(format!("Could not rotate the API XMPP password: {err}"));
    }

    Ok(())
}
