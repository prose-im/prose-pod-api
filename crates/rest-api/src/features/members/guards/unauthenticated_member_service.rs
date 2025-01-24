// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::guards::prelude::*;

impl FromRequestParts<AppState> for service::members::UnauthenticatedMemberService {
    type Rejection = Infallible;

    /// WARN: Use only in initialization routes! Otherwise use `MemberService` directly.
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            Arc::new(state.server_ctl.clone()),
            Arc::new(state.auth_service.clone()),
            Arc::new(state.xmpp_service.clone()),
        ))
    }
}
