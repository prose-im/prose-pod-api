// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for service::dependencies::Uuid {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request_state!(req, service::dependencies::Uuid).map(ToOwned::to_owned)
    }
}
