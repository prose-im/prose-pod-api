// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn test_services_reachability(
    AppState { email_notifier, .. }: &AppState,
) -> Result<(), String> {
    debug!("Testing services reachability…");

    if let Some(email_notifier) = email_notifier {
        match email_notifier.test_connection() {
            Ok(true) => Ok(()),
            Ok(false) => Err("SMTP server unreachable.".to_string()),
            Err(err) => Err(err.to_string()),
        }?;
    }

    Ok(())
}
