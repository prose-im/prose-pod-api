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

use guards::Db;

use migration::MigratorTrait;
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database;
use service::config::Config;
use service::dependencies::Uuid;
#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::{SwaggerUi, Url};

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(rocket: Rocket<Build>, config: &Config) -> Rocket<Build> {
    #[cfg(feature = "swagger-ui")]
    let swagger_ui =
        SwaggerUi::new("/api-docs/swagger-ui/<_..>").config(utoipa_swagger_ui::Config::new([
            Url::with_primary("API v1", "/v1/api-docs/openapi.json", true),
        ]));
    #[cfg(feature = "swagger-ui")]
    let rocket = rocket.mount("/", swagger_ui);

    rocket
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", v1::routes())
        .manage(config.clone())
        .manage(Uuid::from_config(config))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}
