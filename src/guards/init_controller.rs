// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::controllers::init_controller::InitController;

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for InitController<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        Outcome::Success(InitController { db })
    }
}
