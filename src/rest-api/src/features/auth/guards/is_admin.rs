// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::OptionalFromRequestParts;
use service::{auth::UserInfo, members::MemberRepository};

use crate::guards::prelude::*;

/// Checks if the logged in user is an admin.
///
/// It's not perfect, one day we'll replace it with scopes and permissions,
/// but it'll do for now.
pub struct IsAdmin;

impl FromRequestParts<AppState> for IsAdmin {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::auth::is_admin", level = "trace", skip_all)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_info = UserInfo::from_request_parts(parts, state).await?;
        let jid = user_info.jid;

        if MemberRepository::is_admin(&state.db, &jid).await? {
            Ok(Self)
        } else {
            Err(Error::from(error::Forbidden(format!(
                "<{jid}> is not an admin."
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
