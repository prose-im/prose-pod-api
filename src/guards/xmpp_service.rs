// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::prose_xmpp::BareJid;
use service::services::jwt_service::JWT;
use service::services::xmpp_service::{self, XmppServiceContext, XmppServiceInner};

use crate::error::Error;
use crate::request_state;

use super::LazyFromRequest;

pub struct XmppService<'r>(xmpp_service::XmppService<'r>);

impl<'r> Deref for XmppService<'r> {
    type Target = xmpp_service::XmppService<'r>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> Into<xmpp_service::XmppService<'r>> for XmppService<'r> {
    fn into(self) -> xmpp_service::XmppService<'r> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for XmppService<'r> {
    type Error = crate::error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let xmpp_service_inner = try_outcome!(request_state!(req, XmppServiceInner));

        let bare_jid = try_outcome!(BareJid::from_request(req).await);

        let jwt = try_outcome!(JWT::from_request(req).await);
        let prosody_token = match jwt.prosody_token() {
            Ok(prosody_token) => prosody_token,
            Err(err) => {
                debug!("Invalid JWT: {err}");
                return Outcome::Error(Error::Unauthorized.into());
            }
        };

        let ctx = XmppServiceContext {
            bare_jid,
            prosody_token,
        };
        let xmpp_service = xmpp_service::XmppService::new(xmpp_service_inner, ctx);

        Outcome::Success(Self(xmpp_service))
    }
}
