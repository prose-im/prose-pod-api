// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::{xmpp_service, XmppServiceContext, XmppServiceInner};

use crate::error::{self, Error};

use super::{LazyFromRequest, JWT};

pub struct XmppService(xmpp_service::XmppService);

impl Deref for XmppService {
    type Target = xmpp_service::XmppService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<xmpp_service::XmppService> for XmppService {
    fn into(self) -> xmpp_service::XmppService {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for XmppService {
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
            full_jid: jid,
            prosody_token,
        };
        let xmpp_service = xmpp_service::XmppService::new(xmpp_service_inner.inner().clone(), ctx);

        Outcome::Success(Self(xmpp_service))
    }
}
