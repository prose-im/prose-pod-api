// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use service::invitations::InvitationService;

use crate::guards::{prelude::*, UnauthenticatedMemberService};

pub struct UnauthenticatedInvitationService(pub(super) InvitationService);

impl<'r> Deref for UnauthenticatedInvitationService {
    type Target = InvitationService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> Into<InvitationService> for UnauthenticatedInvitationService {
    fn into(self) -> InvitationService {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedInvitationService {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let member_service = try_outcome!(UnauthenticatedMemberService::from_request(req).await).0;

        Outcome::Success(Self(InvitationService::new(member_service)))
    }
}
