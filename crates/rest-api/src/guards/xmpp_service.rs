// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    auth::{auth_service::AuthToken, UserInfo},
    xmpp::{XmppService, XmppServiceContext, XmppServiceInner},
};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for XmppService<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let xmpp_service_inner = try_outcome!(request_state!(req, XmppServiceInner));

        let bare_jid = try_outcome!(UserInfo::from_request(req).await).jid;
        let token = try_outcome!(AuthToken::from_request(req).await);

        let ctx = XmppServiceContext {
            bare_jid,
            prosody_token: token.clone(),
        };
        Outcome::Success(XmppService::new(xmpp_service_inner, ctx))
    }
}
