// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn register_oauth2_client(
    AppState { auth_service, .. }: &AppState,
) -> Result<(), String> {
    debug!("Registering the OAuth 2.0 client…");

    (auth_service.register_oauth2_client().await)
        .map_err(|err| format!("Could not register OAuth 2.0 client: {err}"))?;

    Ok(())
}
