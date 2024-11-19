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

use create_service_accounts::*;
use init_server_config::*;
use register_oauth2_client::*;
use rocket::{Build, Rocket};
use rotate_api_xmpp_password::*;
use run_migrations::*;
use tokio::time::sleep;

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
