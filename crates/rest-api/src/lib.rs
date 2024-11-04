// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

pub mod error;
pub mod features;
pub mod forms;
pub mod guards;
pub mod model;
pub mod responders;

use std::time::Duration;

use error::{Error, ServerConfigNotInitialized};
use guards::Db;
use migration::MigratorTrait;
use rocket::{
    fairing::AdHoc,
    fs::FileServer,
    http::Status,
    {Build, Request, Rocket},
};
use sea_orm_rocket::Database;
use service::{
    config::{AppConfig, Config},
    dependencies::{Notifier, Uuid},
    repositories::ServerConfigRepository,
    services::{
        auth_service::AuthService, jwt_service::JWTService, network_checker::NetworkChecker,
        secrets_store::SecretsStore, server_ctl::ServerCtl, server_manager::ServerManager,
        xmpp_service::XmppServiceInner,
    },
};
use tokio::time::sleep;
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
    secrets_store: SecretsStore,
    network_checker: NetworkChecker,
) -> Rocket<Build> {
    rocket
        .attach(AdHoc::try_on_ignite(
            // NOTE: Fairings run in parallel, which means order is not guaranteed
            //   and race conditions could happen. This fairing runs all the fairings we need,
            //   one after another (since they all depend on the previous one).
            "Sequential fairings",
            |rocket| async {
                match sequential_fairings(&rocket).await {
                    Ok(()) => Ok(rocket),
                    Err(err) => {
                        error!("{err}");
                        Err(rocket)
                    }
                }
            },
        ))
        .mount("/", features::routes())
        .mount("/api-docs", FileServer::from("static/api-docs"))
        .register("/", catchers![default_catcher])
        .manage(Uuid::from_config(&config))
        .manage(config)
        .manage(server_ctl)
        .manage(xmpp_service)
        .manage(auth_service)
        .manage(notifier)
        .manage(jwt_service)
        .manage(secrets_store)
        .manage(network_checker)
}

async fn sequential_fairings(rocket: &Rocket<Build>) -> Result<(), String> {
    run_migrations(rocket).await?;
    // Wait for the XMPP server to finish starting up (1 second should be more than enough)
    sleep(Duration::from_secs(1)).await;
    rotate_api_xmpp_password(rocket).await?;
    init_server_config(rocket).await?;
    create_service_accounts(rocket).await?;
    Ok(())
}

async fn run_migrations(rocket: &Rocket<Build>) -> Result<(), String> {
    let conn = &Db::fetch(&rocket).expect("Db not attached").conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(())
}

async fn rotate_api_xmpp_password(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Rotating Prose Pod API's XMPP password…");

    let server_ctl: &ServerCtl = rocket.state().unwrap();
    let app_config: &AppConfig = rocket.state().unwrap();
    let secrets_store: &SecretsStore = rocket.state().unwrap();

    if let Err(err) =
        ServerManager::rotate_api_xmpp_password(server_ctl, app_config, secrets_store).await
    {
        return Err(format!("Could not rotate the API XMPP password: {err}"));
    }

    Ok(())
}

async fn init_server_config(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Initializing the XMPP server configuration…");

    let db = &Db::fetch(&rocket).unwrap().conn;
    let server_ctl = rocket.state().unwrap();
    let app_config = rocket.state().unwrap();

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!("Not initializing the XMPP server configuration: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!(
                "Could not initialize the XMPP server configuration: {err}"
            ));
        }
    };

    // Apply the server configuration stored in the database
    let server_manager = ServerManager::new(db, app_config, server_ctl, server_config.clone());
    if let Err(err) = server_manager.reload_current().await {
        return Err(format!(
            "Could not initialize the XMPP server configuration: {err}"
        ));
    }

    Ok(())
}

async fn create_service_accounts(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Creating service accounts…");

    let db = &Db::fetch(&rocket).unwrap().conn;
    let server_ctl: &ServerCtl = rocket.state().unwrap();
    let app_config: &AppConfig = rocket.state().unwrap();

    let server_config = match ServerConfigRepository::get(db).await {
        Ok(Some(server_config)) => server_config,
        Ok(None) => {
            info!("Not creating service accounts: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Could not create service accounts: {err}"));
        }
    };

    // Ensure service accounts exist and rotate passwords
    // NOTE: After an update, the Prose Pod API might require more service accounts
    //   than it did when the Prose Pod was initialized. We have to create them before
    //   the Prose Pod API launches.
    let auth_service = rocket.state().unwrap();
    let secrets_store = rocket.state().unwrap();
    if let Err(err) = ServerManager::create_service_accounts(
        &server_config.domain,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
    )
    .await
    {
        return Err(format!("Could not create service accounts: {err}"));
    }

    Ok(())
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> Error {
    error::HTTPStatus(status).into()
}
