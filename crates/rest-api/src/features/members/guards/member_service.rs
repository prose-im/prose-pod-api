// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{
    members::MemberService,
    xmpp::{ServerCtl, XmppService},
};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for MemberService {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
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
