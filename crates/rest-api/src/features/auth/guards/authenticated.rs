// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::auth::UserInfo;

use crate::guards::prelude::*;

/// Checks if a user is logged in.
///
/// Quite often, it's not **mandatory** to add a [`route_layer`] on a [`Router`]
/// because other extractors depend on [`UserInfo`].
/// For example, [`MemberService`] depends on [`UserInfo`] therefore using it in a route
/// already ensures a user is logged in.
/// This is an implementation detail, which might change in the future, which is why
/// one should always make things explicit by adding a [`route_layer`].
///
/// To define it on all routes of a [`Router`], you can use:
///
/// ```rust,no_run
/// use axum::middleware::from_extractor_with_state;
/// use prose_pod_api::AppState;
/// use prose_pod_api::features::auth::guards::Authenticated;
///
/// pub fn router(app_state: AppState) -> axum::Router {
///     axum::Router::new()
///         .route("/example", unimplemented!())
///         .route_layer(from_extractor_with_state::<Authenticated, _>(app_state.clone()))
///         .with_state(app_state)
/// }
/// ```
///
/// To define it on a single route, you can use:
///
/// ```rust,no_run
/// use axum::middleware::from_extractor_with_state;
/// use axum::routing::{delete, get};
/// use prose_pod_api::AppState;
/// use prose_pod_api::features::auth::guards::Authenticated;
///
/// pub fn router(app_state: AppState) -> axum::Router {
///     axum::Router::new()
///         .route("/example", get(unimplemented!()))
///         .route(
///             "/example",
///             delete(unimplemented!())
///                 .route_layer(from_extractor_with_state::<Authenticated, _>(app_state.clone()))
///         )
///         .with_state(app_state)
/// }
/// ```
///
/// [`route_layer`]: axum::Router::route_layer
/// [`Router`]: axum::Router
/// [`UserInfo`]: service::auth::UserInfo
/// [`MemberService`]: service::members::MemberService
pub struct Authenticated;

impl FromRequestParts<AppState> for Authenticated {
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        UserInfo::from_request_parts(parts, state).await?;
        // NOTE: If `UserInfo` could be extracted, it means a user is logged in.
        Ok(Self)
    }
}
