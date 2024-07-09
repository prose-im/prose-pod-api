// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::Request;
use rocket::{outcome::try_outcome, request::Outcome};
use service::repositories::WorkspaceRepository;

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for service::repositories::Workspace {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match WorkspaceRepository::get(db).await {
            Ok(Some(workspace)) => Outcome::Success(workspace),
            Ok(None) => Error::WorkspaceNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}
