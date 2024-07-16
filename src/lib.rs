// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

pub mod error;
pub mod forms;
pub mod guards;
pub mod model;
pub mod responders;
pub mod v1;

use error::Error;
use guards::Db;
use migration::MigratorTrait;
use rocket::{
    fairing::{self, AdHoc},
    fs::{FileServer, NamedFile},
    http::Status,
    {Build, Request, Rocket},
};
use sea_orm_rocket::Database;
use service::{
    config::Config,
    controllers::init_controller::InitController,
    dependencies::{Notifier, Uuid},
    model::ServiceSecretsStore,
    repositories::ServerConfigRepository,
    services::{
        auth_service::AuthService, jwt_service::JWTService, server_ctl::ServerCtl,
        server_manager::ServerManager, xmpp_service::XmppServiceInner,
    },
};
use tracing::{debug, info};

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(
    rocket: Rocket<Build>,
    config: Config,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    notifier: Notifier,
    jwt_service: JWTService,
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
        .register("/", catchers![default_catcher])
        .manage(Uuid::from_config(&config))
        .manage(config)
        .manage(server_ctl)
        .manage(xmpp_service)
        .manage(auth_service)
        .manage(notifier)
        .manage(jwt_service)
        .manage(ServiceSecretsStore::default())
}

async fn server_config_init(rocket: Rocket<Build>) -> fairing::Result {
    debug!("Initializing the XMPP server configuration…");

    let db = &Db::fetch(&rocket).unwrap().conn;
    let server_ctl = rocket.state().unwrap();
    let app_config = rocket.state().unwrap();

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!(
                "Not reloading the XMPP server: {}",
                error::ServerConfigNotInitialized
            );
            return Ok(rocket);
        }
        Err(err) => {
            error!("Not reloading the XMPP server: {err}");
            return Ok(rocket);
        }
    };

    // Apply the server configuration stored in the database
    let server_manager = ServerManager::new(db, app_config, server_ctl, server_config.clone());
    if let Err(err) = server_manager.reload_current().await {
        error!("Could not initialize the XMPP server configuration: {err}");
        return Err(rocket);
    }

    // Ensure service accounts exist and rotate passwords
    // NOTE: After an update, the Prose Pod API might require more service accounts
    //   than it did when the Prose Pod was initialized. We have to create them before
    //   the Prose Pod API launches.
    let auth_service = rocket.state().unwrap();
    let secrets_store = rocket.state().unwrap();
    if let Err(err) = InitController::create_service_accounts(
        &server_config.domain,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
    )
    .await
    {
        error!("Could not initialize the XMPP server configuration: {err}");
        return Err(rocket);
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
        .map_err(|e| {
            error::NotFound {
                reason: format!("{e}"),
            }
            .into()
        })
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> Error {
    error::HTTPStatus(status).into()
}
