// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::response::Redirect;
use axum::routing::*;
use axum::{extract::State, middleware::from_extractor_with_state};
use service::secrets::SecretsStore;
use service::xmpp::{ServerCtl, ServerManager};

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
    State(AppState { db, .. }): State<AppState>,
    server_ctl: ServerCtl,
    secrets_store: SecretsStore,
) -> Result<Redirect, Error> {
    tracing::debug!("Doing factory reset…");

    tracing::debug!("Erasing user data from the server…");
    server_ctl.delete_all_data().await?;
    tracing::debug!("Resetting the server…");
    ServerManager::reset_server_config(&db, &server_ctl, &secrets_store).await?;
    tracing::debug!("Resetting the API’s database…");
    tracing::debug!("Resetting the API’s configuration file…");

    tracing::debug!("Factory reset done.");

    tokio::task::spawn(async {
        tracing::debug!("Restarting the API…");
    });

    Ok(Redirect::to("/"))
}
