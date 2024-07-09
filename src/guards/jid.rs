// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::{prose_xmpp::BareJid, services::jwt_service::JWT};

use crate::error::{self};

use super::LazyFromRequest;

pub struct JID(BareJid);

impl Deref for JID {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for JID {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = try_outcome!(JWT::from_request(req).await);
        match jwt.jid() {
            Ok(jid) => Outcome::Success(Self(jid)),
            Err(err) => {
                debug!("Invalid JWT: {err}");
                Outcome::Error(Self::Error::Unauthorized.into())
            }
        }
    }
}
