// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::UnauthenticatedMemberService;

use crate::guards::prelude::*;

#[async_trait::async_trait]
impl FromRequestParts<AppState> for service::invitations::InvitationService {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let member_service = UnauthenticatedMemberService::from_request_parts(parts, state).await?;

        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.uuid_gen.clone()),
            member_service,
        ))
    }
}
