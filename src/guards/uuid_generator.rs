// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use service::dependencies::Uuid;

use super::prelude::*;

pub struct UuidGenerator(Uuid);

impl Deref for UuidGenerator {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UuidGenerator {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request_state!(req, Uuid).map(|state| Self(state.clone()))
    }
}
