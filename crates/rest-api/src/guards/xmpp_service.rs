// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    auth::{auth_service::AuthToken, UserInfo},
    xmpp::{XmppService, XmppServiceContext, XmppServiceInner},
};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for XmppService {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let xmpp_service_inner = try_outcome!(request_state!(req, XmppServiceInner));

        let bare_jid = try_outcome!(UserInfo::from_request(req).await).jid;
        let token = try_outcome!(AuthToken::from_request(req).await);

        let ctx = XmppServiceContext {
            bare_jid,
            prosody_token: token.clone(),
        };
        Outcome::Success(XmppService::new(Arc::new(xmpp_service_inner.clone()), ctx))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for XmppService {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let bare_jid = UserInfo::from_request_parts(parts, state).await?.jid;
        let token = AuthToken::from_request_parts(parts, state).await?;

        let ctx = XmppServiceContext {
            bare_jid,
            prosody_token: token.clone(),
        };
        Ok(XmppService::new(Arc::new(state.xmpp_service.clone()), ctx))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for XmppServiceInner {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.xmpp_service.clone())
    }
}
