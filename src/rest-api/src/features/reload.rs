// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::State, http::StatusCode, middleware::from_extractor_with_state, routing::post,
};
use service::auth::IsAdmin;

use crate::{AppState, MinimalAppState};

pub(crate) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/reload", post(reload_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

pub(crate) fn factory_reset_router(minimal_app_state: MinimalAppState) -> axum::Router {
    axum::Router::new()
        .route("/reload", post(reload_route))
        .with_state(minimal_app_state)
}

async fn reload_route(
    State(MinimalAppState {
        ref lifecycle_manager,
        ..
    }): State<MinimalAppState>,
) -> StatusCode {
    // FIXME: This is bad. I (@RemiBardon) know. It’s just a temporary shortcut
    //   until I implement https://github.com/prose-im/prose-pod-api/issues/357.
    //   The current Pod API architecture doesn’t allow easy access to the
    //   Pod Server API during this stage, I have no choice without a huge
    //   rewrite.
    {
        use service::reqwest;
        let client = reqwest::Client::new();
        tracing::debug!("Reloading the Server…");
        match client
            .post("http://prose-pod-server:8080/lifecycle/reload")
            .send()
            .await
        {
            Ok(_response) => {}
            Err(err) => {
                tracing::error!("{err:#}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    lifecycle_manager.set_restarting();
    StatusCode::ACCEPTED
}
