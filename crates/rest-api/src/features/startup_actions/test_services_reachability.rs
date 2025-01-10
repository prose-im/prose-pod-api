// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::debug;

use crate::AppState;

pub async fn test_services_reachability(
    AppState { email_notifier, .. }: &AppState,
) -> Result<(), String> {
    debug!("Testing services reachability…");

    match email_notifier.test_connection() {
        Ok(true) => Ok(()),
        Ok(false) => Err("SMTP server unreachable.".to_string()),
        Err(err) => Err(err.to_string()),
    }
}
