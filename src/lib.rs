// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

mod prosody_config;
pub mod prosody_ctl;
pub mod server_ctl;
pub mod v1;

use migration::MigratorTrait;
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database;
use utoipa_swagger_ui::{Config, SwaggerUi, Url};

pub mod pool;
use pool::Db;

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    let swagger_ui = SwaggerUi::new("/api-docs/swagger-ui/<_..>").config(Config::new([
        Url::with_primary("API v1", "/v1/api-docs/openapi.json", true),
    ]));

    rocket
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", v1::routes())
        .mount("/", swagger_ui)
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}
