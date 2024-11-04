// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::repositories::WorkspaceRepository;

use crate::{features::init::WorkspaceNotInitialized, guards::prelude::*};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for service::model::Workspace {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match WorkspaceRepository::get(db).await {
            Ok(Some(model)) => Outcome::Success(model),
            Ok(None) => Error::from(WorkspaceNotInitialized).into(),
            Err(err) => Error::from(err).into(),
        }
    }
}
