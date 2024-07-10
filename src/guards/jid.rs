// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{prose_xmpp::BareJid, services::jwt_service::JWT};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for BareJid {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = try_outcome!(JWT::from_request(req).await);
        match jwt.jid() {
            Ok(jid) => Outcome::Success(jid),
            Err(err) => Outcome::Error(Error::Unauthorized(format!("Invalid JWT: {err}")).into()),
        }
    }
}
