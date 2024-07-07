// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::services::xmpp_service::{self, XmppServiceContext, XmppServiceInner};

use crate::error::{self, Error};

use super::{LazyFromRequest, JWT};

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
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let xmpp_service_inner = try_outcome!(req
            .guard::<&State<XmppServiceInner>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<XmppServiceInner>` from a request."
                        .to_string(),
                }
            )));
        let jwt = try_outcome!(JWT::from_request(req).await);
        let jid = match jwt.jid() {
            Ok(jid) => jid,
            Err(err) => return Outcome::Error(err.into()),
        };
        let prosody_token = match jwt.prosody_token() {
            Ok(prosody_token) => prosody_token,
            Err(err) => return Outcome::Error(err.into()),
        };
        let ctx = XmppServiceContext {
            bare_jid: jid,
            prosody_token,
        };
        let xmpp_service = xmpp_service::XmppService::new(xmpp_service_inner, ctx);

        Outcome::Success(Self(xmpp_service))
    }
}
