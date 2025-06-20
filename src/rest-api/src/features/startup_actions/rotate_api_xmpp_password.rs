// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::xmpp::server_manager;
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn rotate_api_xmpp_password(
    AppState {
        server_ctl,
        app_config,
        secrets_store,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Rotating Prose Pod API's XMPP password…");

    let ref app_config = app_config.read().unwrap().clone();

    if let Err(err) =
        server_manager::rotate_api_xmpp_password(server_ctl, app_config, secrets_store).await
    {
        return Err(format!("Could not rotate the API XMPP password: {err}"));
    }

    Ok(())
}
