// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use service::{server_config::ServerConfigRepository, xmpp::ServerManager};

use crate::{features::init::ServerConfigNotInitialized, guards::Db};

pub async fn init_server_config(rocket: &Rocket<Build>) -> Result<(), String> {
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
