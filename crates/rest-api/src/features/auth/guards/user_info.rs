// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::auth::{auth_service::AuthToken, AuthService, UserInfo};

use crate::guards::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UserInfo {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
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
