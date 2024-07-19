// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{controllers::invitation_controller::InvitationController, dependencies};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for InvitationController<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let uuid_gen = try_outcome!(request_state!(req, dependencies::Uuid));

        Outcome::Success(InvitationController { db, uuid_gen })
    }
}
