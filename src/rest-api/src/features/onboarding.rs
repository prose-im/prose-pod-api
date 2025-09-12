// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::State;
use axum::middleware::from_extractor_with_state;
use axum::routing::get;
use axum::Json;
use service::{
    auth::Authenticated,
    onboarding::{self, OnboardingStepsStatuses},
};

use crate::AppState;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/onboarding-steps", get(onboarding_steps_route))
        .route_layer(from_extractor_with_state::<Authenticated, _>(
            app_state.clone(),
        ))
        .with_state(app_state)
}

async fn onboarding_steps_route(
    State(AppState { ref db, .. }): State<AppState>,
) -> Json<OnboardingStepsStatuses> {
    Json(onboarding::get_steps_statuses(db).await)
}
