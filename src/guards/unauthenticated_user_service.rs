// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::services::user_service::UserService;
use service::services::{
    auth_service::AuthService, server_ctl::ServerCtl, xmpp_service::XmppServiceInner,
};

use crate::request_state;

use super::LazyFromRequest;

pub struct UnauthenticatedUserService<'r>(pub(super) UserService<'r>);

impl<'r> Deref for UnauthenticatedUserService<'r> {
    type Target = UserService<'r>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> Into<UserService<'r>> for UnauthenticatedUserService<'r> {
    fn into(self) -> UserService<'r> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedUserService<'r> {
    type Error = crate::error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let server_ctl = try_outcome!(request_state!(req, ServerCtl));
        let auth_service = try_outcome!(request_state!(req, AuthService));
        let xmpp_service_inner = try_outcome!(request_state!(req, XmppServiceInner));

        // // Make sure the Prose Pod is initialized, as we can't add or remove users otherwise.
        // // TODO: Check that the Prose Pod is initialized another way (this doesn't cover all cases)
        // let db = try_outcome!(database_connection(req).await);
        // match ServerConfigRepository::get(db).await {
        //     Ok(Some(_)) => {}
        //     Ok(None) => return Error::ServerConfigNotInitialized.into(),
        //     Err(err) => return Error::DbErr(err).into(),
        // }

        Outcome::Success(Self(UserService::new(
            server_ctl,
            auth_service,
            xmpp_service_inner,
        )))
    }
}
