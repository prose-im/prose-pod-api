// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{rejection::QueryRejection, FromRequestParts, OptionalFromRequestParts, Query},
    http::request::Parts,
};
use serdev::de::DeserializeOwned;

#[derive(Debug, Clone, Copy, Default)]
pub struct OptionalQuery<T>(pub T);

impl<T, S> OptionalFromRequestParts<S> for OptionalQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QueryRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(t)) => Ok(Some(Self(t))),
            Err(QueryRejection::FailedToDeserializeQueryString(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
