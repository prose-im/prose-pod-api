// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::network_checks::NetworkChecker;

use crate::guards::prelude::*;

#[async_trait::async_trait]
impl FromRequestParts<AppState> for NetworkChecker {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.network_checker.clone())
    }
}
