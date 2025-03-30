// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::{self, File};
use std::io::Write;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::{extract::State, middleware::from_extractor_with_state};
use axum::{routing::*, Json};
use axum_extra::either::Either;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use serde::{Deserialize, Serialize};
use service::app_config::CONFIG_FILE_PATH;
use service::secrets::SecretsStore;
use service::xmpp::{ServerCtl, ServerManager};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{Error, ErrorCode};
use crate::AppState;

use super::auth::guards::IsAdmin;

lazy_static! {
    static ref FACTORY_RESET_CONFIRMATION_CODE: RwLock<Option<String>> = RwLock::default();
}

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", delete(factory_reset_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

#[derive(Debug, Serialize, Deserialize)]
struct FactoryResetConfirmation {
    pub confirmation: String,
}

async fn factory_reset_route(
    State(AppState {
        db,
        app_config,
        lifecycle_manager,
        ..
    }): State<AppState>,
    server_ctl: ServerCtl,
    secrets_store: SecretsStore,
    req: Option<Json<FactoryResetConfirmation>>,
) -> Result<Either<(StatusCode, Json<FactoryResetConfirmation>), StatusCode>, Error> {
    match req {
        None => {
            // Generate a new 16-characters long string.
            let confirmation = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>();
            // Store the code for later confirmation.
            // WARN: This means two people can’t ask for a factory reset concurrently… but who cares?
            *FACTORY_RESET_CONFIRMATION_CODE.write().await = Some(confirmation.clone());
            // Return the code to the user.
            return Ok(Either::E1((
                StatusCode::ACCEPTED,
                Json(FactoryResetConfirmation { confirmation }),
            )));
        }
        Some(Json(FactoryResetConfirmation {
            confirmation: validation,
        })) => {
            if Some(validation) != *FACTORY_RESET_CONFIRMATION_CODE.read().await {
                return Err(Error::new(
                    ErrorCode::BAD_REQUEST,
                    "Invalid confirmation code".to_owned(),
                    None,
                    vec![format!(
                        "Call `DELETE /` to get a new confirmation code."
                    )],
                    vec![],
                ));
            }
        }
    }

    warn!("Doing factory reset…");

    debug!("Resetting the server…");
    ServerManager::reset_server_config(&db, &server_ctl, &app_config, &secrets_store).await?;

    debug!("Erasing user data from the server…");
    server_ctl.delete_all_data().await?;

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

    info!("Restarting the API…");
    lifecycle_manager.set_restarting();

    Ok(Either::E2(StatusCode::RESET_CONTENT))
}

pub async fn restart_guard(
    State(AppState {
        lifecycle_manager, ..
    }): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if lifecycle_manager.is_restarting() {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            // NOTE: A second should be enough, the API usually takes around 60ms to start.
            .header("Retry-After", 1)
            .body("The API is restarting.".into())
            .unwrap();
    }
    next.run(request).await
}
