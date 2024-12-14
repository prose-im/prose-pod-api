// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use service::{auth::AuthService, server_config::ServerConfigRepository};
use tracing::{debug, info};

use crate::{features::init::ServerConfigNotInitialized, guards::Db, AppState};

pub async fn register_oauth2_client(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Registering the OAuth 2.0 client…");

    let db = &Db::fetch(&rocket).unwrap().conn;

    // NOTE: If the server config is not initialized, it means the Prosody configuration won't be reloaded at startup.
    //   In this situation, `mod_http_oauth2` is not activated therefore we can't create the OAuth 2.0 client.
    match ServerConfigRepository::get(db).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            info!("Not registering the OAuth 2.0 client: {ServerConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Could not register the OAuth 2.0 client: {err}"));
        }
    };

    let auth_service: &AuthService = rocket.state().unwrap();
    auth_service
        .register_oauth2_client()
        .await
        .map_err(|err| format!("Could not register OAuth 2.0 client: {err}"))?;

    Ok(())
}

pub async fn register_oauth2_client_axum(_app_state: &AppState) -> Result<(), String> {
    todo!()
}
