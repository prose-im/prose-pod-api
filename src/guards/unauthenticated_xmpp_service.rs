// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::config::Config;
use service::{xmpp_service, XmppServiceContext, XmppServiceInner};

use crate::error::{self, Error};

use super::LazyFromRequest;

/// Same as `XmppService` but uses Prose Pod API's JID instead of the logged in user's.
///
/// WARN: Use only in places where the route doesn't support authentication like when
///   accepting or rejecting a workspace invitation. Otherwise, use `XmppService`.
pub struct UnauthenticatedXmppService(xmpp_service::XmppService);

impl Deref for UnauthenticatedXmppService {
    type Target = xmpp_service::XmppService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<xmpp_service::XmppService> for UnauthenticatedXmppService {
    fn into(self) -> xmpp_service::XmppService {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedXmppService {
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
        let config = try_outcome!(req
            .guard::<&State<Config>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<Config>` from a request.".to_string(),
                }
            )));
        let xmpp_service_ctx = XmppServiceContext {
            bare_jid: config.api_jid(),
        };
        let xmpp_service =
            xmpp_service::XmppService::new(xmpp_service_inner.inner().clone(), xmpp_service_ctx);

        Outcome::Success(Self(xmpp_service))
    }
}
