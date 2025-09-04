// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::auth::AuthService {
    type Rejection = Infallible;

    #[tracing::instrument(name = "req::extract::auth_service", level = "trace", skip_all)]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.auth_service.clone())
    }
}
