// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::auth::auth_service::AuthToken;

use crate::guards::prelude::*;

impl FromRequestParts<AppState> for service::auth::UserInfo {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::auth::user_info", level = "trace", skip_all)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = AuthToken::from_request_parts(parts, state).await?;

        state
            .auth_service
            .get_user_info(token)
            .await
            .map_err(|err| {
                Error::from(error::Unauthorized(format!(
                    "Could not get user info from token: {err}"
                )))
            })
    }
}
