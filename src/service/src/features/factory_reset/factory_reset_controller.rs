// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::{self, File};
use std::io::Write;

use anyhow::Context;
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use secrecy::ExposeSecret;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::app_config::CONFIG_FILE_PATH;
use crate::auth::errors::InvalidCredentials;
use crate::auth::{AuthService, Credentials};
use crate::secrets::SecretsStore;
use crate::util::either::Either;
use crate::xmpp::{server_manager, ServerCtl};
use crate::AppConfig;

lazy_static! {
    static ref FACTORY_RESET_CONFIRMATION_CODE: RwLock<Option<String>> = RwLock::default();
}

#[instrument(
    name = "factory_reset_controller::get_confirmation_code",
    level = "trace",
    skip_all, fields(jid = credentials.jid.to_string()),
)]
pub async fn get_confirmation_code(
    credentials: &Credentials,
    auth_service: &AuthService,
) -> Result<String, InvalidCredentials> {
    // Test password.
    // NOTE: This cannot be used to brute-force a user’s password since
    //   the request is already authenticated using a valid OAuth 2.0
    //   token for that user.
    // TODO: Use a method that does not create a new token.
    if (auth_service
        .log_in(&credentials.jid, &credentials.password)
        .await)
        .is_err()
    {
        return Err(InvalidCredentials);
    }

    // Generate a random 16-characters-long string.
    let confirmation = crate::auth::util::strong_random_password(16)
        .expose_secret()
        .to_owned();

    {
        // Store the code for later confirmation.
        // WARN: This means two people can’t ask for a factory reset concurrently…
        //   but who cares?
        *FACTORY_RESET_CONFIRMATION_CODE.write().await = Some(confirmation.clone());
    }

    Ok(confirmation)
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid confirmation code.")]
pub struct InvalidConfirmationCode;

#[instrument(
    name = "factory_reset_controller::perform_factory_reset",
    level = "trace",
    skip_all
)]
pub async fn perform_factory_reset(
    confirmation: String,
    db: DatabaseConnection,
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    secrets_store: &SecretsStore,
) -> Result<(), Either<InvalidConfirmationCode, anyhow::Error>> {
    if Some(confirmation) != *FACTORY_RESET_CONFIRMATION_CODE.read().await {
        return Err(Either::E1(InvalidConfirmationCode));
    }

    warn!("Performing a factory reset…");

    debug!("Resetting the server…");
    (server_manager::reset_server_config(&db, server_ctl, app_config, secrets_store).await)
        .context("Could not reset server config")
        .map_err(Either::E2)?;

    debug!("Erasing user data from the server…");
    (server_ctl.delete_all_data().await)
        .context("Could not erase all server data")
        .map_err(Either::E2)?;

    debug!("Resetting the API’s database…");
    // Close the database connection to make sure SeaORM
    // doesn’t write to it after we empty the file.
    (db.close().await)
        .context("Could not close the database connection")
        .map_err(Either::E2)?;
    // Then empty the database file.
    // NOTE: We don’t just revert database migrations to ensure nothing remains.
    let database_url = (app_config.api.databases.main.url)
        .strip_prefix("sqlite://")
        .context("Database URL should start with `sqlite://`")
        .map_err(Either::E2)?;
    // NOTE: `File::create` truncates the file if it exists.
    File::create(database_url)
        .context(format!("Could not reset API database at <{database_url}>"))
        .map_err(Either::E2)?;

    debug!("Resetting the API’s configuration file…");
    let config_file_path = CONFIG_FILE_PATH.as_path();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(config_file_path)
        .context(format!(
            "Could not reset API config file at <{path}>: Cannot open",
            path = config_file_path.display(),
        ))
        .map_err(Either::E2)?;
    let bootstrap_config = r#"# Prose Pod API configuration file
# Template: https://github.com/prose-im/prose-pod-system/blob/master/templates/prose.toml
# All keys: https://github.com/prose-im/prose-pod-api/blob/master/src/service/src/features/app_config/mod.rs
"#;
    file.write_all(bootstrap_config.as_bytes())
        .context(format!(
            "Could not reset API config file at <{path}>: Cannot write",
            path = config_file_path.display(),
        ))
        .map_err(Either::E2)?;

    info!("Factory reset done.");
    Ok(())
}
