// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::OptionalFromRequestParts;
use service::auth::{IsAdmin, UserInfo};

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for IsAdmin {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::auth::is_admin", level = "trace", skip_all)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_info = UserInfo::from_request_parts(parts, state).await?;

        if user_info.is_admin() {
            Ok(Self)
        } else {
            Err(Error::from(error::Forbidden(format!(
                "<{jid}> is not an admin.",
                jid = user_info.jid
            ))))
        }
    }
}

impl OptionalFromRequestParts<AppState> for IsAdmin {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <Self as FromRequestParts<AppState>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}
