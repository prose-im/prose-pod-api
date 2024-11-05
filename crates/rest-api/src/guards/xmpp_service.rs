// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    features::{
        auth::JWT,
        xmpp::{XmppService, XmppServiceContext, XmppServiceInner},
    },
    model::BareJid,
};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for XmppService<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let xmpp_service_inner = try_outcome!(request_state!(req, XmppServiceInner));

        let bare_jid = try_outcome!(BareJid::from_request(req).await);

        let jwt = try_outcome!(JWT::from_request(req).await);
        let prosody_token = match jwt.prosody_token() {
            Ok(prosody_token) => prosody_token,
            Err(err) => {
                return Outcome::Error(
                    Error::from(error::Unauthorized(format!("Invalid JWT: {err}"))).into(),
                );
            }
        };

        let ctx = XmppServiceContext {
            bare_jid,
            prosody_token,
        };
        Outcome::Success(XmppService::new(xmpp_service_inner, ctx))
    }
}
