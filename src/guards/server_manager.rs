// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use prose_pod_core::repositories::MemberRepository;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest, UnauthenticatedServerManager, JID as JIDGuard};

pub struct ServerManager<'r>(UnauthenticatedServerManager<'r>);

impl<'r> From<UnauthenticatedServerManager<'r>> for ServerManager<'r> {
    fn from(inner: UnauthenticatedServerManager<'r>) -> Self {
        Self(inner)
    }
}

impl<'r> Deref for ServerManager<'r> {
    type Target = UnauthenticatedServerManager<'r>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerManager<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        let jid = try_outcome!(JIDGuard::from_request(req).await);
        match MemberRepository::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                debug!("<{}> is not an admin", jid.to_string());
                return Error::Unauthorized.into();
            }
            Err(e) => return Error::DbErr(e).into(),
        }

        UnauthenticatedServerManager::from_request(req)
            .await
            .map(Self)
    }
}
