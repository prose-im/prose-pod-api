// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use service::{app_config::DatabaseConfig, sea_orm};

pub async fn db_conn(
    config: &DatabaseConfig,
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    db_conn_with(config, |_| {}).await
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
    additional_options(&mut options);
    sea_orm::Database::connect(options.to_owned()).await
}
