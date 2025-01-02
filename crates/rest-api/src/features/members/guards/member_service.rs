// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    members::MemberService,
    xmpp::{ServerCtl, XmppService},
};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for MemberService {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let server_ctl = try_outcome!(request_state!(req, ServerCtl));
        let xmpp_service = try_outcome!(XmppService::from_request(req).await);

        try_outcome!(check_caller_is_admin(req, Some(db)).await);

        Outcome::Success(MemberService::new(
            Arc::new(db.clone()),
            Arc::new(server_ctl.clone()),
            Arc::new(xmpp_service),
        ))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for MemberService {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let xmpp_service = XmppService::from_request_parts(parts, state).await?;

        Ok(MemberService::new(
            Arc::new(state.db.clone()),
            Arc::new(state.server_ctl.clone()),
            Arc::new(xmpp_service),
        ))
    }
}
