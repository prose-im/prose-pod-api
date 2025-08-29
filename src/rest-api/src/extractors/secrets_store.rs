// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

impl FromRequestParts<AppState> for service::secrets::SecretsStore {
    type Rejection = Infallible;

    #[tracing::instrument(name = "req::extract::secrets_store", level = "trace", skip_all, err)]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.base.secrets_store.clone())
    }
}
