// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::model::ServerConfig;
use service::repositories::ServerConfigRepository;

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for ServerConfig {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        match ServerConfigRepository::get(db).await {
            Ok(Some(model)) => Outcome::Success(model),
            Ok(None) => Error::from(error::ServerConfigNotInitialized).into(),
            Err(err) => Error::from(err).into(),
        }
    }
}
