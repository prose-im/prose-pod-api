// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{Build, Rocket};
use service::xmpp::ServerCtl;
use tracing::debug;

/// Wait for the XMPP server to finish starting up.
pub async fn wait_for_server(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Waiting for XMPP server to start…");

    let server_ctl: &ServerCtl = rocket.state().unwrap();
    server_ctl
        .wait_until_ready()
        .await
        .map_err(|err| format!("Error while waiting for XMPP server to start: {err}"))
}
