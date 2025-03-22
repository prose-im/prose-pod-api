// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::{debug, instrument};

use crate::AppState;

/// Wait for the XMPP server to finish starting up.
#[instrument(level = "trace", skip_all, err)]
pub async fn wait_for_server(AppState { server_ctl, .. }: &AppState) -> Result<(), String> {
    debug!("Waiting for XMPP server to start…");

    server_ctl
        .wait_until_ready()
        .await
        .map_err(|err| format!("Error while waiting for XMPP server to start: {err}"))
}
