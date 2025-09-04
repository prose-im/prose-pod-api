// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod errors;
mod extractors;
mod routes;

use axum::routing::MethodRouter;

use crate::AppState;

pub use self::routes::*;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/init/first-account",
            MethodRouter::new()
                .put(init_first_account_route)
                .head(is_first_account_created_route),
        )
        .with_state(app_state)
}
