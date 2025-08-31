// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod extractors;

use axum::middleware::from_extractor_with_state;
use axum::routing::put;
use axum::{extract::State, response::NoContent};
use service::{auth::IsAdmin, licensing::License};

use crate::{error::Error, AppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/licensing/license", put(set_license))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

async fn set_license(
    State(AppState {
        ref license_service,
        ..
    }): State<AppState>,
    license: License,
) -> Result<NoContent, Error> {
    match license_service.install_license(license) {
        Ok(()) => Ok(NoContent),
        Err(err) => Err(Error::from(err)),
    }
}
