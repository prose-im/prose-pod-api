// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use service::{
    app_config::ConfigDatabase, errors::DbErr, sea_orm::DatabaseConnection, MigratorTrait as _,
};
use tracing::*;

pub async fn db_conn(config: &ConfigDatabase) -> DatabaseConnection {
    prose_pod_api::util::database::db_conn(
        &config,
        Some(|opts: &mut service::sea_orm::ConnectOptions| {
            opts.sqlx_logging_level(FromStr::from_str("WARN").unwrap());
        }),
    )
    .await
    .expect("Database connection failed")
}

pub async fn run_migrations(conn: &DatabaseConnection) -> Result<(), DbErr> {
    debug!("Running database migrations before creating the Rocket…");
    service::Migrator::up(conn, None).await
}
