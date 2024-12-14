// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod create_service_accounts;
mod init_server_config;
mod register_oauth2_client;
mod rotate_api_xmpp_password;
mod run_migrations;

use std::time::Duration;

use rocket::{Build, Rocket};
use tokio::time::sleep;

use crate::AppState;

use self::create_service_accounts::*;
use self::init_server_config::*;
use self::register_oauth2_client::*;
use self::rotate_api_xmpp_password::*;
use self::run_migrations::*;

pub async fn sequential_fairings(rocket: &Rocket<Build>) -> Result<(), String> {
    run_migrations(rocket).await?;
    // Wait for the XMPP server to finish starting up (1 second should be more than enough)
    sleep(Duration::from_secs(1)).await;
    rotate_api_xmpp_password(rocket).await?;
    init_server_config(rocket).await?;
    register_oauth2_client(rocket).await?;
    create_service_accounts(rocket).await?;
    Ok(())
}

pub async fn on_startup(app_state: &AppState) -> Result<(), String> {
    run_migrations_axum(app_state).await?;
    // Wait for the XMPP server to finish starting up (1 second should be more than enough)
    sleep(Duration::from_secs(1)).await;
    rotate_api_xmpp_password_axum(app_state).await?;
    init_server_config_axum(app_state).await?;
    register_oauth2_client_axum(app_state).await?;
    create_service_accounts_axum(app_state).await?;
    Ok(())
}
