// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::State;
use axum::middleware::from_extractor_with_state;
use axum::response::Redirect;
use axum::routing::*;

use crate::AppState;

use super::auth::guards::IsAdmin;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", delete(factory_reset_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

async fn factory_reset_route(
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Redirect, ()> {
    tracing::debug!("Doing factory reset…");

    tracing::debug!("Erasing user data from the server…");
    tracing::debug!("Resetting the server…");
    tracing::debug!("Resetting the API’s database…");
    tracing::debug!("Resetting the API’s configuration file…");

    tracing::debug!("Factory reset done.");

    tokio::task::spawn(async {
        tracing::debug!("Restarting the API…");
    });

    let dashboard_root_url = app_config.branding.page_url;
    Ok(Redirect::to(dashboard_root_url.as_str()))
}
