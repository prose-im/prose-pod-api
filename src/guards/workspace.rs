// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::workspace;
use rocket::Request;
use rocket::{outcome::try_outcome, request::Outcome};
use sea_orm_rocket::Connection;
use service::Query;

use crate::error::{self, Error};

use super::{Db, LazyFromRequest};

// TODO: Make it so we can call `workspace.field` directly
// instead of `workspace.model.field`.
#[repr(transparent)]
pub struct Workspace(workspace::Model);

impl Workspace {
    pub fn model(self) -> workspace::Model {
        self.0
    }
}

impl Deref for Workspace {
    type Target = workspace::Model;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for Workspace {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));

        match Query::workspace(db).await {
            Ok(Some(workspace)) => Outcome::Success(Self(workspace)),
            Ok(None) => Error::WorkspaceNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}
