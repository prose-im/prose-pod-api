// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{dependencies, invitations::InvitationController};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for InvitationController {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let uuid_gen = try_outcome!(request_state!(req, dependencies::Uuid));

        Outcome::Success(InvitationController {
            db: Arc::new(db.clone()),
            uuid_gen: Arc::new(uuid_gen.clone()),
        })
    }
}
