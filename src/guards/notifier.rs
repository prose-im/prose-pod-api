// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{outcome::try_outcome, request::Outcome, Request};
use service::services::notifier::Notifier;

use crate::error;
use crate::guards::util::check_caller_is_admin;
use crate::request_state;

use super::{util::database_connection, LazyFromRequest};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for Notifier<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        try_outcome!(check_caller_is_admin(req, Some(db)).await);

        let notifier = try_outcome!(request_state!(req, service::dependencies::Notifier));
        let config = try_outcome!(request_state!(req, service::config::Config));

        Outcome::Success(Self::new(db, notifier, &config.branding))
    }
}
