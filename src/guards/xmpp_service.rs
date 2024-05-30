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

use super::{LazyFromRequest, JID as JIDGuard};

pub struct XmppService(xmpp_service::XmppService);

impl Deref for XmppService {
    type Target = xmpp_service::XmppService;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        let jid = try_outcome!(JIDGuard::from_request(req).await);
        let xmpp_service_ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
        };
        let xmpp_service =
            xmpp_service::XmppService::new(xmpp_service_inner.inner().clone(), xmpp_service_ctx);

        Outcome::Success(Self(xmpp_service))
    }
}

impl XmppService {}
