// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::util::either::Either;
use tracing::{debug, instrument};

use crate::{error::InvalidServerConfiguration, AppState};

/// Wait for the XMPP server to finish starting up.
#[instrument(level = "trace", skip_all, err)]
pub async fn wait_for_server(
    AppState {
        prose_pod_server_service,
        ..
    }: &AppState,
) -> Result<Either<(), InvalidServerConfiguration>, String> {
    debug!("Waiting for XMPP server to start…");

    match prose_pod_server_service.wait_until_ready().await {
        Ok(()) => Ok(Either::E1(())),
        Err(Either::E1(err)) => Ok(Either::E2(InvalidServerConfiguration(err))),
        Err(Either::E2(err)) => Err(format!(
            "Error while waiting for XMPP server to start: {err}"
        )),
    }
}
