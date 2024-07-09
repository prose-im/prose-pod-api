// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{outcome::try_outcome, request::Outcome, Request};
use service::{prose_xmpp::BareJid, services::jwt_service::JWT};

use super::LazyFromRequest;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for BareJid {
    type Error = crate::error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = try_outcome!(JWT::from_request(req).await);
        match jwt.jid() {
            Ok(jid) => Outcome::Success(jid),
            Err(err) => {
                debug!("Invalid JWT: {err}");
                Outcome::Error(Self::Error::Unauthorized.into())
            }
        }
    }
}
