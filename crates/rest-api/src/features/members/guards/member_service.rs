// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::xmpp::XmppService;

use crate::guards::prelude::*;

#[async_trait::async_trait]
impl FromRequestParts<AppState> for service::members::MemberService {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let xmpp_service = XmppService::from_request_parts(parts, state).await?;

        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.server_ctl.clone()),
            Arc::new(xmpp_service),
        ))
    }
}
