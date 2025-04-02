// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod add_workspace_to_team;
mod create_service_accounts;
mod init_server_config;
mod register_oauth2_client;
mod rotate_api_xmpp_password;
mod run_migrations;
mod test_services_reachability;
mod wait_for_server;

use tracing::{instrument, trace};

use crate::{error::DETAILED_ERROR_REPONSES, AppState};

use self::add_workspace_to_team::*;
use self::create_service_accounts::*;
use self::init_server_config::*;
use self::register_oauth2_client::*;
use self::rotate_api_xmpp_password::*;
use self::run_migrations::*;
use self::test_services_reachability::*;
use self::wait_for_server::*;

#[instrument(level = "trace", skip_all, err)]
pub async fn run_startup_actions(app_state: &AppState) -> Result<(), String> {
    trace!("Running startup actions…");

    DETAILED_ERROR_REPONSES.store(
        app_state.app_config.debug.detailed_error_responses,
        std::sync::atomic::Ordering::Relaxed,
    );

    run_migrations(app_state).await?;
    test_services_reachability(app_state).await?;
    wait_for_server(app_state).await?;
    rotate_api_xmpp_password(app_state).await?;
    init_server_config(app_state).await?;
    register_oauth2_client(app_state).await?;
    create_service_accounts(app_state).await?;
    add_workspace_to_team(app_state).await?;

    Ok(())
}
