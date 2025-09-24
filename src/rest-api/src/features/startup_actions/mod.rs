// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod backfill_database;
mod create_service_accounts;
mod db_configure;
mod db_run_migrations;
mod init_server_config;
mod migrate_workspace_vcard;
mod register_oauth2_client;
mod rotate_api_xmpp_password;
mod start_cron_tasks;
mod test_services_reachability;
mod update_rosters;
mod validate_app_config_changes;
mod wait_for_server;

use tracing::{info, instrument, trace, warn};

use crate::{error::DETAILED_ERROR_REPONSES, AppState};

use self::backfill_database::*;
use self::create_service_accounts::*;
use self::db_configure::*;
use self::db_run_migrations::*;
use self::init_server_config::*;
use self::migrate_workspace_vcard::*;
use self::register_oauth2_client::*;
use self::rotate_api_xmpp_password::*;
use self::start_cron_tasks::*;
use self::test_services_reachability::*;
use self::update_rosters::*;
use self::validate_app_config_changes::*;
use self::wait_for_server::*;

macro_rules! run_step_macro {
    ($app_state:ident, $app_config:ident) => {
        macro_rules! run_step {
            ($step:ident) => {
                if ($app_config.debug.skip_startup_actions).contains(stringify!($step)) {
                    warn!(
                        "Not running startup step '{}': Step marked to skip in the app configuration.",
                        stringify!($step),
                    );
                } else {
                    $step(&$app_state).await?;
                }
            };
        }
    };
}

#[instrument(level = "trace", skip_all, err)]
pub async fn run_startup_actions(app_state: AppState) -> Result<(), String> {
    trace!("Running startup actions…");

    let ref app_config = app_state.app_config;
    DETAILED_ERROR_REPONSES.store(
        app_config.debug.detailed_error_responses,
        std::sync::atomic::Ordering::Relaxed,
    );
    if app_config.debug.log_config_at_startup {
        info!("app_config: {app_config:#?}");
    }

    {
        run_step_macro!(app_state, app_config);

        run_step!(db_configure);
        run_step!(db_run_migrations);
        let temp = true;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        run_step!(test_services_reachability);
        run_step!(wait_for_server);
        run_step!(rotate_api_xmpp_password);
        run_step!(validate_app_config_changes);
        run_step!(init_server_config);
        run_step!(register_oauth2_client);
        run_step!(create_service_accounts);
        run_step!(migrate_workspace_vcard);
        run_step!(start_cron_tasks);
    }

    // Some actions won’t prevent the API from running properly so let’s not
    // make startup longer because of it.
    async fn run_remaining(
        app_state @ AppState { app_config, .. }: &AppState,
    ) -> Result<(), String> {
        run_step_macro!(app_state, app_config);

        run_step!(backfill_database);
        // NOTE: `update_rosters` should run after `backfill_database`
        //   as the latter can add team members.
        run_step!(update_rosters);

        Ok(())
    }
    tokio::spawn(
        async move {
            if let Err(err) = run_remaining(&app_state).await {
                warn!("{}", crate::StartupError(err));
            }
            (app_state.base.lifecycle_manager).set_startup_actions_finished();
        },
        // FIXME: For some reason, this breaks behavior tests.
        // .in_current_span(),
    );

    Ok(())
}
