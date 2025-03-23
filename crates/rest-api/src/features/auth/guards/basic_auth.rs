// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use axum::http::header::AUTHORIZATION;
use http_auth_basic::Credentials;
use secrecy::SecretString;
use service::models::BareJid;

use crate::guards::prelude::*;

pub struct BasicAuth {
    pub jid: BareJid,
    pub password: SecretString,
}

impl FromRequestParts<AppState> for BasicAuth {
    type Rejection = error::Error;

    #[tracing::instrument(
        name = "req::auth::authenticate::basic",
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
        let creds = Credentials::from_header(auth.to_string()).map_err(|err| {
            Error::from(error::Unauthorized(format!(
                "The '{AUTHORIZATION}' header is not a valid Basic authentication string: {err}"
            )))
        })?;
        let jid = BareJid::from_str(&creds.user_id).map_err(|err| {
            Error::from(error::Unauthorized(format!(
                "The JID present in the '{AUTHORIZATION}' header could not be parsed to a valid JID: {err}"
            )))
        })?;
        Ok(Self {
            jid,
            password: creds.password.into(),
        })
    }
}
