// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{auth::JWT, models::BareJid};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for BareJid {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = try_outcome!(JWT::from_request(req).await);
        match jwt.jid() {
            Ok(jid) => Outcome::Success(jid),
            Err(err) => Outcome::Error(
                Error::from(error::Unauthorized(format!("Invalid JWT: {err}"))).into(),
            ),
        }
    }
}
