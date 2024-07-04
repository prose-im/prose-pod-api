// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use prose_pod_core::dependencies::Uuid;
use rocket::{
    request::{FromRequest, Outcome},
    Request, State,
};

use crate::error::{self, Error};

pub struct UuidGenerator(Uuid);

impl Deref for UuidGenerator {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UuidGenerator {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        req.guard::<&State<Uuid>>()
            .await
            .map(|state| Self(state.inner().clone()))
            .map_error(|(status, _)| {
                (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<Uuid>` from a request.".to_string(),
                    },
                )
            })
    }
}
