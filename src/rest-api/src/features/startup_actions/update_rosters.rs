// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::{debug, instrument};

use crate::AppState;

/// NOTE: Rosters resynchronization (for teams) is an expensive operation
///   (O(n^2)), therefore the API debounces it. If a team member is added but
///   the API is restarted before the debounce timeout (e.g. in tests), rosters
///   become inconsistent. This forces a resynchronization at startup.
#[instrument(level = "trace", skip_all, err)]
pub async fn update_rosters(AppState { server_ctl, .. }: &AppState) -> Result<(), String> {
    debug!("Updating rosters…");

    if let Err(err) = server_ctl.force_rosters_sync().await {
        return Err(format!("Could not update rosters: {err}"));
    }

    Ok(())
}
