// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use service::{
    server_config::ServerConfigRepository,
    xmpp::{ServerCtl, ServerManager},
    AppConfig,
};
use tracing::{debug, info};

use crate::{features::init::ServerConfigNotInitialized, guards::Db, AppState};

pub async fn init_server_config(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Initializing the XMPP server configuration…");

    let db = &Db::fetch(&rocket).unwrap().conn;
    let server_ctl: &ServerCtl = rocket.state().unwrap();
    let app_config: &AppConfig = rocket.state().unwrap();

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
    let server_manager = ServerManager::new(
        Arc::new(db.clone()),
        Arc::new(app_config.clone()),
        Arc::new(server_ctl.clone()),
        server_config.clone(),
    );
    if let Err(err) = server_manager.reload_current().await {
        return Err(format!(
            "Could not initialize the XMPP server configuration: {err}"
        ));
    }

    Ok(())
}

pub async fn init_server_config_axum(
    AppState {
        db,
        server_ctl,
        app_config,
        ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Initializing the XMPP server configuration…");

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
    let server_manager = ServerManager::new(
        Arc::new(db.clone()),
        Arc::new(app_config.clone()),
        Arc::new(server_ctl.clone()),
        server_config.clone(),
    );
    if let Err(err) = server_manager.reload_current().await {
        return Err(format!(
            "Could not initialize the XMPP server configuration: {err}"
        ));
    }

    Ok(())
}
