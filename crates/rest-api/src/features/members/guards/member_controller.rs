// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{members::MemberController, xmpp::XmppService};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for MemberController {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let xmpp_service = try_outcome!(XmppService::from_request(req).await);

        Outcome::Success(MemberController::new(
            Arc::new(db.clone()),
            Arc::new(xmpp_service),
        ))
    }
}
