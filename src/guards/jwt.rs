// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;

use entity::model;
use log::debug;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use secrecy::Secret;
use service::{AuthService, JWT_JID_KEY, JWT_PROSODY_TOKEN_KEY};

use crate::error::{self, Error};

use super::LazyFromRequest;

pub struct JWT {
    pub claims: BTreeMap<String, String>,
}

impl JWT {
    pub fn try_from(jwt: &str, auth_service: &AuthService) -> Result<Self, String> {
        match auth_service.verify(jwt) {
            Ok(claims) => Ok(Self { claims }),
            Err(e) => Err(format!("JWT is could not be verified: {e}")),
        }
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for JWT {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            debug!("No 'Authorization' header found");
            return Error::Unauthorized.into();
        };
        let Some(token) = auth.strip_prefix("Bearer ") else {
            debug!("The 'Authorization' header does not start with 'Bearer '");
            return Error::Unauthorized.into();
        };

        let auth_service =
            try_outcome!(req
                .guard::<&State<AuthService>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<AuthService>` from a request.".to_string(),
                    }
                )));

        match Self::try_from(token, auth_service) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(err) => {
                debug!("The provided JWT is invalid: {err}");
                return Error::Unauthorized.into();
            }
        }
    }
}

impl JWT {
    pub fn jid(&self) -> Result<model::JID, <Self as LazyFromRequest>::Error> {
        let Some(jid) = self.claims.get(JWT_JID_KEY) else {
            debug!("The provided JWT does not contain a '{JWT_JID_KEY}' claim");
            Err(Error::Unauthorized)?
        };
        model::JID::try_from(jid.clone()).map_err(|e| {
            debug!("The JID present in the JWT could not be parsed to a valid JID: {e}");
            Error::Unauthorized
        })
    }
    pub fn prosody_token(&self) -> Result<Secret<String>, <Self as LazyFromRequest>::Error> {
        let Some(token) = self.claims.get(JWT_PROSODY_TOKEN_KEY) else {
            debug!("The provided JWT does not contain a '{JWT_PROSODY_TOKEN_KEY}' claim");
            Err(Error::Unauthorized)?
        };
        Ok(token.to_owned().into())
    }
}
