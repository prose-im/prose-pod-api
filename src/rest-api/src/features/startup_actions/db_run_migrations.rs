// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::MigratorTrait as _;
use tracing::instrument;

use crate::AppState;

#[instrument(level = "trace", skip_all, err)]
pub async fn db_run_migrations(AppState { db, .. }: &AppState) -> Result<(), String> {
    let _ = service::Migrator::up(&db.write, None).await;
    Ok(())
}
