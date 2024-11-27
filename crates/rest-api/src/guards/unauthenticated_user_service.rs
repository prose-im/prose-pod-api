// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, sync::Arc};

use service::{
    auth::AuthService,
    members::UserService,
    xmpp::{ServerCtl, XmppServiceInner},
};

use super::prelude::*;

/// WARN: Use only in initialization routes! Otherwise use `UserService` directly.
#[derive(Clone)]
pub struct UnauthenticatedUserService(pub UserService);

impl Deref for UnauthenticatedUserService {
    type Target = UserService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<UserService> for UnauthenticatedUserService {
    fn into(self) -> UserService {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedUserService {
    type Error = error::Error;

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
            Arc::new(server_ctl.clone()),
            Arc::new(auth_service.clone()),
            Arc::new(xmpp_service_inner.clone()),
        )))
    }
}
