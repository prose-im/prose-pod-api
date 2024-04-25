// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

pub mod error;
pub mod forms;
pub mod guards;
pub mod responders;
pub mod v1;

use error::Error;
use guards::Db;

use migration::MigratorTrait;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{FileServer, NamedFile};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database;
use service::config::Config;
use service::dependencies::Uuid;

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(rocket: Rocket<Build>, config: &Config) -> Rocket<Build> {
    rocket
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", v1::routes())
        .mount("/api-docs", FileServer::from("static/api-docs"))
        .mount("/api-docs", routes![redoc])
        .manage(config.clone())
        .manage(Uuid::from_config(config))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[get("/redoc")]
async fn redoc() -> Result<NamedFile, Error> {
    NamedFile::open("static/api-docs/redoc.html")
        .await
        .map_err(|e| Error::NotFound {
            reason: format!("{e}"),
        })
}
