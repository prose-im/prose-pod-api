// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::sea_orm::ConnectionTrait as _;
use tracing::{debug, instrument};

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn db_configure(AppState { db, .. }: &AppState) -> Result<(), String> {
    debug!("Configuring the database…");

    // Allow simultaneous reads with one writer.
    db.write
        .execute_unprepared("PRAGMA journal_mode=WAL;")
        .await
        .map_err(|err| format!("Could not set `journal_mode=WAL`: {err}"))?;

    Ok(())
}
