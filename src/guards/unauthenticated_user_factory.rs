// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::repositories::ServerConfigRepository;
use service::{AuthService, ServerCtl, XmppServiceInner};

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest, UserFactory};

pub struct UnauthenticatedUserFactory<'r>(UserFactory<'r>);

impl<'r> Deref for UnauthenticatedUserFactory<'r> {
    type Target = UserFactory<'r>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> Into<UserFactory<'r>> for UnauthenticatedUserFactory<'r> {
    fn into(self) -> UserFactory<'r> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UnauthenticatedUserFactory<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let server_ctl =
            try_outcome!(req
                .guard::<&State<ServerCtl>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<ServerCtl>` from a request.".to_string(),
                    }
                )));
        let auth_service =
            try_outcome!(req
                .guard::<&State<AuthService>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<AuthService>` from a request.".to_string(),
                    }
                )));
        let xmpp_service_inner = try_outcome!(req
            .guard::<&State<XmppServiceInner>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<XmppServiceInner>` from a request."
                        .to_string(),
                }
            )));

        // Make sure the Prose Pod is initialized, as we can't add or remove users otherwise.
        // TODO: Check that the Prose Pod is initialized another way (this doesn't cover all cases)
        let db = try_outcome!(database_connection(req).await);
        match ServerConfigRepository::get(db).await {
            Ok(Some(_)) => {}
            Ok(None) => return Error::ServerConfigNotInitialized.into(),
            Err(err) => return Error::DbErr(err).into(),
        }

        Outcome::Success(Self(UserFactory::new(
            server_ctl,
            auth_service,
            xmpp_service_inner,
        )))
    }
}
