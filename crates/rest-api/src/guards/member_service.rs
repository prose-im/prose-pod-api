// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::MemberService;

use super::{prelude::*, UnauthenticatedMemberService};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for MemberService {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        try_outcome!(check_caller_is_admin(req, None).await);

        UnauthenticatedMemberService::from_request(req)
            .await
            .map(|s| s.0)
    }
}
