// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::guards::prelude::*;

#[async_trait::async_trait]
impl FromRequestParts<AppState> for service::init::InitService {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: Arc::new(state.db.clone()),
        })
    }
}
