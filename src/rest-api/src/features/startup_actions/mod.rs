// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod backfill_database;
mod db_configure;
mod db_run_migrations;
mod init_server_config;
mod start_cron_tasks;
mod test_services_reachability;
mod validate_app_config_changes;
mod wait_for_server;

use service::util::either::Either;
use tracing::{info, instrument, trace, warn, Instrument as _};

use crate::{
    error::{InvalidServerConfiguration, DETAILED_ERROR_REPONSES},
    AppState,
};

use self::backfill_database::*;
use self::db_configure::*;
use self::db_run_migrations::*;
use self::init_server_config::*;
use self::start_cron_tasks::*;
use self::test_services_reachability::*;
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
            ($step:ident or $default:expr) => {
                async {
                    if ($app_config.debug.skip_startup_actions).contains(stringify!($step)) {
                        warn!(
                            "Not running startup step '{}': Step marked to skip in the app configuration.",
                            stringify!($step),
                        );
                        $default
                    } else {
                        $step(&$app_state).await
                    }
                }
            };
        }
    };
}

#[instrument(level = "trace", skip_all, err)]
#[must_use]
pub async fn run_startup_actions(
    app_state: AppState,
) -> Result<Either<(), InvalidServerConfiguration>, String> {
    trace!("Running startup actions…");

    let ref app_config = app_state.app_config;
    DETAILED_ERROR_REPONSES.store(
        app_config.debug.detailed_error_responses,
        std::sync::atomic::Ordering::Relaxed,
    );
    if app_config.debug.log_config_at_startup {
        info!("app_config: {app_config:#?}");
    }

    let startup_res = {
        run_step_macro!(app_state, app_config);

        run_step!(db_configure);
        run_step!(db_run_migrations);
        run_step!(test_services_reachability);
        run_step!(validate_app_config_changes);
        let res = run_step!(wait_for_server or Ok(Either::E1(()))).await?;
        if let Either::E1(()) = res {
            run_step!(init_server_config);
        }
        run_step!(start_cron_tasks);
        res
    };

    // Some actions won’t prevent the API from running properly so let’s not
    // make startup longer because of it.
    async fn run_remaining(
        app_state @ AppState { app_config, .. }: &AppState,
        startup_succeeded: bool,
    ) -> Result<(), String> {
        run_step_macro!(app_state, app_config);

        if startup_succeeded {
            run_step!(backfill_database);
        }

        Ok(())
    }
    let startup_succeeded = matches!(startup_res, Either::E1(()));
    tokio::spawn(
        async move {
            if let Err(err) = run_remaining(&app_state, startup_succeeded).await {
                warn!("{}", crate::StartupError(err));
            }
            (app_state.base.lifecycle_manager).set_startup_actions_finished();
        }
        .in_current_span(),
    );

    Ok(startup_res)
}
