// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::workspace::WorkspaceService {
    type Rejection = Infallible;

    #[tracing::instrument(
        name = "req::extract::workspace_service",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.workspace_service.clone())
    }
}
