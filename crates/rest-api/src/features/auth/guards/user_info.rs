// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::auth::{auth_service::AuthToken, AuthService, UserInfo};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UserInfo {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let token = try_outcome!(AuthToken::from_request(req).await);

        let auth_service = try_outcome!(request_state!(req, AuthService));

        match auth_service.get_user_info(token).await {
            Ok(user_info) => Outcome::Success(user_info),
            Err(err) => Outcome::Error(
                Error::from(error::Forbidden(format!(
                    "Could not get user info from token: {err}"
                )))
                .into(),
            ),
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for UserInfo {
    type Rejection = error::Error;

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
                Error::from(error::Forbidden(format!(
                    "Could not get user info from token: {err}"
                )))
            })
    }
}
