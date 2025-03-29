// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::{self, File};
use std::io::Write;

use axum::response::Redirect;
use axum::routing::*;
use axum::{extract::State, middleware::from_extractor_with_state};
use service::app_config::CONFIG_FILE_PATH;
use service::secrets::SecretsStore;
use service::xmpp::{ServerCtl, ServerManager};
use tracing::{debug, info, warn};

use crate::error::Error;
use crate::AppState;

use super::auth::guards::IsAdmin;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", delete(factory_reset_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

async fn factory_reset_route(
    State(AppState { db, app_config, .. }): State<AppState>,
    server_ctl: ServerCtl,
    secrets_store: SecretsStore,
) -> Result<Redirect, Error> {
    warn!("Doing factory reset…");

    debug!("Erasing user data from the server…");
    server_ctl.delete_all_data().await?;

    debug!("Resetting the server…");
    ServerManager::reset_server_config(&db, &server_ctl, &secrets_store).await?;

    debug!("Resetting the API’s database…");
    // Close the database connection to make sure SeaORM
    // doesn’t write to it after we empty the file.
    db.close().await?;
    // Then empty the database file.
    // NOTE: We don’t just revert database migrations to ensure nothing remains.
    let database_url = (app_config.databases.main.url)
        .strip_prefix("sqlite://")
        .expect("Database URL should start with `sqlite://`");
    // NOTE: `File::create` truncates the file if it exists.
    File::create(database_url).expect(&format!("Could not reset API database at <{database_url}>"));

    debug!("Resetting the API’s configuration file…");
    let config_file_path = CONFIG_FILE_PATH.as_path();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(config_file_path)
        .expect(&format!(
            "Could not reset API config file at <{}>: Cannot open",
            config_file_path.display(),
        ));
    let bootstrap_config = r#"# Prose Pod API configuration file
# Example: https://github.com/prose-im/prose-pod-system/blob/master/Prose-example.toml
# All keys: https://github.com/prose-im/prose-pod-api/blob/master/src/service/src/features/app_config/mod.rs
"#;
    file.write_all(bootstrap_config.as_bytes()).expect(&format!(
        "Could not reset API config file at <{}>: Cannot write",
        config_file_path.display(),
    ));

    info!("Factory reset done.");

    tokio::task::spawn(async {
        warn!("Restarting the API…");
    });

    Ok(Redirect::to("/"))
}
