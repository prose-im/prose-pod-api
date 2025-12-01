// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::xmpp::XmppServiceContext {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::auth::xmpp_service_context", level = "trace", skip_all)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        use service::auth::{AuthToken, UserInfo};

        let bare_jid = UserInfo::from_request_parts(parts, state).await?.jid;
        let auth_token = AuthToken::from_request_parts(parts, state).await?;

        Ok(Self {
            bare_jid,
            auth_token,
        })
    }
}
