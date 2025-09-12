// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{auth::AuthToken, util::either::Context};
use tracing::trace;

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::auth::UserInfo {
    type Rejection = error::Error;

    #[tracing::instrument(name = "req::auth::user_info", level = "trace", skip_all)]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Read cache to avoid unnecessary recomputations.
        // NOTE: On a local run, this extractor seems to take around 5ms to run.
        //   It doesn’t seem much, but this function can be called multiple
        //   times *per request* resulting in unnecessary delay. In addition,
        //   every call to `AuthService::get_user_info` results in at least one
        //   call to the XMPP server and at least one call to the database. If
        //   one of those is already under heavy load (which was not the case in
        //   our local test run), this extractor will take even longer (and
        //   increase said load).
        //   Caching avoids all of that and a cache hit takes around 25µs
        //   (likely O(1)) which is a non-negligible improvement (>200x faster).
        // NOTE: Unless it becomes an issue, we won’t add a higher level cache
        //   to avoid recomputations on repeated calls. Such cache, if
        //   misimplemented, could result in security issues (wrong
        //   role/privileges). If we ever do implement such cache, we MUST make
        //   sure said cache expires after a short time.
        if let Some(cache) = parts.extensions.get::<Option<Self>>() {
            trace!("Cache hit.");
            return match cache {
                Some(user_info) => Ok(user_info.to_owned()),
                None => todo!(),
            };
        }

        // Get user info from auth token.
        let token = AuthToken::from_request_parts(parts, state).await?;
        let res = (state.auth_service.get_user_info(token, &state.db).await)
            .context("Could not get user info from token")
            .map_err(Error::from);

        // Cache value to avoid recomputations next time.
        (parts.extensions).insert::<Option<Self>>(res.as_ref().ok().cloned());
        trace!("Cache stored.");

        res
    }
}
