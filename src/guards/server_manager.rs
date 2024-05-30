// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use sea_orm_rocket::Connection;
use service::Query;

use crate::error::{self, Error};

use super::{Db, LazyFromRequest, UnauthenticatedServerManager, JID as JIDGuard};

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
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));

        let jid = try_outcome!(JIDGuard::from_request(req).await);
        match Query::is_admin(db, &jid.node).await {
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
