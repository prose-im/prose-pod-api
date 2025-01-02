// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for service::dependencies::Uuid {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        request_state!(req, service::dependencies::Uuid).map(ToOwned::to_owned)
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for service::dependencies::Uuid {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.uuid_gen.clone())
    }
}
