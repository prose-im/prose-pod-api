// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    auth::UserInfo, members::MemberServiceContext, util::ConcurrentTaskRunner, xmpp::XmppService,
};

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::members::MemberService {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::extract::member_service", level = "trace", skip_all, err)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let xmpp_service = XmppService::from_request_parts(parts, state).await?;
        let user_info = UserInfo::from_request_parts(parts, state).await?;
        let ctx = MemberServiceContext {
            bare_jid: user_info.jid,
        };

        Ok(Self::new(
            Arc::new(state.db.clone()),
            Arc::new(state.server_ctl.clone()),
            Arc::new(xmpp_service),
            ConcurrentTaskRunner::default(&state.app_config),
            ctx,
        ))
    }
}
