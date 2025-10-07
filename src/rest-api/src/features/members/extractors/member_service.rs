// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::members::MemberService {
    type Rejection = Infallible;

    #[tracing::instrument(name = "req::extract::member_service", level = "trace", skip_all, err)]
    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            state.user_repository.clone(),
            state.user_application_service.clone(),
            state.app_config.server_domain().to_owned(),
            state.xmpp_service.clone(),
            state.auth_service.clone(),
            None,
            &state.app_config.api.member_enriching,
        ))
    }
}
