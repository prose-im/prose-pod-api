// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::notifications::{dependencies, Notifier};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for Notifier<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        try_outcome!(check_caller_is_admin(req, Some(db)).await);

        let notifier = try_outcome!(request_state!(req, dependencies::Notifier));
        let config = try_outcome!(request_state!(req, service::AppConfig));

        Outcome::Success(Self::new(db, notifier, &config.branding))
    }
}
