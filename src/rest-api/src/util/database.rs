// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use anyhow::Context;
use service::{
    app_config::DatabaseConfig,
    models::DatabaseRwConnectionPools,
    sea_orm::{self, ConnectionTrait},
};

pub async fn db_conn(
    read_config: &DatabaseConfig,
    write_config: &DatabaseConfig,
) -> Result<DatabaseRwConnectionPools, anyhow::Error> {
    tracing::debug!(
        "Opening database write connection at `{url}`…",
        url = &write_config.url
    );
    let write_pool = db_conn_with(write_config, |_| {}).await?;

    // Configure the database before opening the read connection pool as most
    // configuration would not be visible to the other pool until a restart.
    configure_db(&write_pool).await?;

    tracing::debug!(
        "Opening database read connection at `{url}`…",
        url = &read_config.url
    );
    let read_pool = db_conn_with(read_config, |_| {}).await?;

    Ok(DatabaseRwConnectionPools {
        read: read_pool,
        write: write_pool,
    })
}

pub async fn db_conn_with(
    config: &DatabaseConfig,
    additional_options: impl FnOnce(&mut sea_orm::ConnectOptions) -> (),
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    let mut options = sea_orm::ConnectOptions::new(config.url.clone());
    options
        .max_connections(config.max_connections as u32)
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .sqlx_logging(config.sqlx_logging);
    if let Some(min_connections) = config.min_connections {
        options.min_connections(min_connections);
    }
    if let Some(idle_timeout) = config.idle_timeout {
        options.idle_timeout(Duration::from_secs(idle_timeout));
    }
    if let Some(acquire_timeout) = config.acquire_timeout {
        options.acquire_timeout(Duration::from_secs(acquire_timeout));
    }
    additional_options(&mut options);
    sea_orm::Database::connect(options.to_owned()).await
}

#[tracing::instrument(level = "trace", skip_all, err)]
async fn configure_db(db: &sea_orm::DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::debug!("Configuring the database…");

    // Allow simultaneous reads with one writer.
    db.execute_unprepared("PRAGMA journal_mode=WAL;")
        .await
        .context("Could not set `journal_mode=WAL`")?;

    Ok(())
}
