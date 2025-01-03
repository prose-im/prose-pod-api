// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

#[axum::async_trait]
impl FromRequestParts<AppState> for service::notifications::Notifier {
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
