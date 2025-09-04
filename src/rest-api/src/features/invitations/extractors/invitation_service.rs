// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::UnauthenticatedMemberService;

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::invitations::InvitationService {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let member_service = UnauthenticatedMemberService::from_request_parts(parts, state).await?;

        Ok(Self::new(
            state.db.clone(),
            state.uuid_gen.clone(),
            member_service,
        ))
    }
}
