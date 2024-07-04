// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)
//
// Based on:
// - <https://github.com/SeaQL/sea-orm/blob/53ec0e15488ea71d5da71b1d3ac3f2d9a3b2b97d/examples/rocket_example/api/src/pool.rs>

use async_trait::async_trait;
use sea_orm_rocket::{
    rocket::{figment::Figment, request::Outcome, Request},
    Config, Connection, Database,
};
use service::sea_orm;
use std::time::Duration;

use crate::error::Error;

#[derive(Database, Debug)]
#[database("data")]
pub struct Db(SeaOrmPool);

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
    type Error = sea_orm::DbErr;

    type Connection = sea_orm::DatabaseConnection;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config = figment.extract::<Config>().unwrap();
        let mut options: sea_orm::ConnectOptions = config.url.into();
        options
            .max_connections(config.max_connections as u32)
            .min_connections(config.min_connections.unwrap_or_default())
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .sqlx_logging(config.sqlx_logging);
        if let Some(idle_timeout) = config.idle_timeout {
            options.idle_timeout(Duration::from_secs(idle_timeout));
        }
        let conn = sea_orm::Database::connect(options).await?;

        Ok(SeaOrmPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}

pub(super) async fn database_connection<'r, 'a>(
    req: &'r Request<'a>,
) -> Outcome<&'r sea_orm::DatabaseConnection, Error> {
    req.guard::<Connection<'_, Db>>()
        .await
        .map(|conn| conn.into_inner())
        .map_error(|(status, err)| (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr)))
}
