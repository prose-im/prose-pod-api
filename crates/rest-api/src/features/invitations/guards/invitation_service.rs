// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{
    dependencies, invitations::InvitationService, members::UnauthenticatedMemberService,
};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for InvitationService {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let uuid_gen = try_outcome!(request_state!(req, dependencies::Uuid));
        let member_service = try_outcome!(UnauthenticatedMemberService::from_request(req).await);

        Outcome::Success(Self::new(
            Arc::new(db.clone()),
            Arc::new(uuid_gen.clone()),
            member_service,
        ))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for InvitationService {
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
