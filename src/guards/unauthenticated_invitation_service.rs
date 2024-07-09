// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::services::invitation_service::InvitationService;

use crate::error;

use super::{LazyFromRequest, UnauthenticatedUserService};

pub struct UnauthenticatedInvitationService<'r>(InvitationService<'r>);

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

        // // Make sure the Prose Pod is initialized, as we can't add or remove users otherwise.
        // // TODO: Check that the Prose Pod is initialized another way (this doesn't cover all cases)
        // let db = try_outcome!(database_connection(req).await);
        // match ServerConfigRepository::get(db).await {
        //     Ok(Some(_)) => {}
        //     Ok(None) => return Error::ServerConfigNotInitialized.into(),
        //     Err(err) => return Error::DbErr(err).into(),
        // }

        Outcome::Success(Self(InvitationService::new(user_service)))
    }
}
