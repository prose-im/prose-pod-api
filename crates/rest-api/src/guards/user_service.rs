// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::UserService;

use super::{prelude::*, UnauthenticatedUserService};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UserService<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        try_outcome!(check_caller_is_admin(req, None).await);

        UnauthenticatedUserService::from_request(req)
            .await
            .map(|s| s.0)
    }
}
