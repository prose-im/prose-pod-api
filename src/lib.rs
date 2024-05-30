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
use guards::{Db, UnauthenticatedServerManager};

use log::{debug, info};
use migration::MigratorTrait;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{FileServer, NamedFile};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database;
use service::config::Config;
use service::dependencies::Uuid;
use service::{Query, ServerCtl};

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(
    rocket: Rocket<Build>,
    config: Config,
    server_ctl: ServerCtl,
) -> Rocket<Build> {
    rocket
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .attach(AdHoc::try_on_ignite(
            "Server config init",
            server_config_init,
        ))
        .mount("/", v1::routes())
        .mount("/api-docs", FileServer::from("static/api-docs"))
        .mount("/api-docs", routes![redoc])
        .manage(Uuid::from_config(&config))
        .manage(config)
        .manage(server_ctl)
}

async fn server_config_init(rocket: Rocket<Build>) -> fairing::Result {
    debug!("Initializing the XMPP server configuration…");

    let db = &Db::fetch(&rocket).unwrap().conn;
    let server_ctl = rocket.state().unwrap();
    let app_config = rocket.state().unwrap();

    match Query::server_config(db).await {
        Ok(Some(server_config)) => {
            let server_manager =
                UnauthenticatedServerManager::new(db, app_config, server_ctl, server_config);
            if let Err(err) = server_manager.reload_current() {
                error!("Could not initialize the XMPP server configuration: {err}");
            }
        }
        Ok(None) => info!(
            "Not reloading the XMPP server: {}",
            Error::ServerConfigNotInitialized
        ),
        Err(err) => error!("Not reloading the XMPP server: {err}"),
    }

    Ok(rocket)
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
