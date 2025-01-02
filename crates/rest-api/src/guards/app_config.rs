// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::AppConfig;

use crate::guards::prelude::*;

#[axum::async_trait]
impl FromRequestParts<AppState> for AppConfig {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.app_config.clone())
    }
}
