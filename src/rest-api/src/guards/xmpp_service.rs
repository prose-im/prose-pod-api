// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    auth::{auth_service::AuthToken, UserInfo},
    xmpp::{XmppService, XmppServiceContext, XmppServiceInner},
};

use super::prelude::*;

impl FromRequestParts<AppState> for XmppService {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::extract::xmpp_service", level = "trace", skip_all, err)]
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

impl FromRequestParts<AppState> for XmppServiceInner {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.xmpp_service.clone())
    }
}
