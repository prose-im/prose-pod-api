// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::members::UnauthenticatedMemberService {
    type Rejection = Infallible;

    /// WARN: Use only in initialization routes! Otherwise use `MemberService` directly.
    #[tracing::instrument(
        name = "req::extract::unauthenticated_member_service",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            state.server_ctl.clone(),
            state.auth_service.clone(),
            state.license_service.clone(),
            state.xmpp_service.clone(),
        ))
    }
}
