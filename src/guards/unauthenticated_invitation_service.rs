// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use service::services::invitation_service::InvitationService;

use super::{prelude::*, UnauthenticatedUserService};

pub struct UnauthenticatedInvitationService<'r>(pub(super) InvitationService<'r>);

impl<'r> Deref for UnauthenticatedInvitationService<'r> {
    type Target = InvitationService<'r>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> Into<InvitationService<'r>> for UnauthenticatedInvitationService<'r> {
    fn into(self) -> InvitationService<'r> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedInvitationService<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_service = try_outcome!(UnauthenticatedUserService::from_request(req).await).0;

        Outcome::Success(Self(InvitationService::new(user_service)))
    }
}
