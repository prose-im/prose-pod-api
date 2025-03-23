// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::header::AUTHORIZATION;
use secrecy::SecretString;

use crate::guards::prelude::*;

const PREFIX: &'static str = "Bearer ";

impl FromRequestParts<AppState> for service::auth::auth_service::AuthToken {
    type Rejection = error::Error;

    #[tracing::instrument(
        name = "req::auth::authenticate::bearer",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = parts.headers.get(AUTHORIZATION) else {
            return Err(Error::from(error::Unauthorized(format!(
                "No '{AUTHORIZATION}' header found."
            ))));
        };
        let auth = auth.to_str().map_err(|err| {
            Error::from(error::Unauthorized(format!(
                "Bad '{AUTHORIZATION}' header value: {err}"
            )))
        })?;
        let Some(token) = auth.strip_prefix(PREFIX) else {
            return Err(Error::from(error::Unauthorized(format!(
                "The '{AUTHORIZATION}' header does not start with '{PREFIX}'."
            ))));
        };

        Ok(Self(SecretString::new(token.to_string())))
    }
}
