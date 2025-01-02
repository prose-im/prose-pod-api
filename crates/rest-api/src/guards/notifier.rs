// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::notifications::{dependencies, Notifier};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for Notifier {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        try_outcome!(check_caller_is_admin(req, Some(db)).await);

        let notifier = try_outcome!(request_state!(req, dependencies::Notifier));
        let config = try_outcome!(request_state!(req, service::AppConfig));

        Outcome::Success(Self::new(
            Arc::new(db.clone()),
            Arc::new(notifier.clone()),
            Arc::new(config.branding.clone()),
        ))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Notifier {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.notifier.clone()),
            Arc::new(state.app_config.branding.clone()),
        ))
    }
}
