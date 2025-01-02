// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::header::AUTHORIZATION;
use secrecy::SecretString;
use service::auth::auth_service::AuthToken;

use crate::guards::prelude::*;

const PREFIX: &'static str = "Bearer ";

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for AuthToken {
    type Error = error::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            return Error::from(error::Unauthorized(
                "No 'Authorization' header found.".to_string(),
            ))
            .into();
        };
        let Some(token) = auth.strip_prefix(PREFIX) else {
            return Error::from(error::Unauthorized(format!(
                "The 'Authorization' header does not start with '{PREFIX}'."
            )))
            .into();
        };

        Outcome::Success(AuthToken(SecretString::new(token.to_string())))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthToken {
    type Rejection = error::Error;

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

        Ok(AuthToken(SecretString::new(token.to_string())))
    }
}
