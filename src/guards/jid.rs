// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;
use std::ops::Deref;

use entity::model;
use jwt::VerifyWithKey;
use log::debug;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

use crate::error::{self, Error};

use super::JWTKey;

pub const JWT_JID_KEY: &'static str = "jid";

pub struct JID(model::JID);

impl Deref for JID {
    type Target = model::JID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JID {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            return Outcome::Error(Error::Unauthorized.into());
        };
        let Some(jwt) = auth.strip_prefix("Bearer ") else {
            debug!("The 'Authorization' header does not start with 'Bearer '");
            return Outcome::Error(Error::Unauthorized.into());
        };

        let jwt_key = try_outcome!(req
            .guard::<&State<JWTKey>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<JWTKey>` from a request.".to_string(),
                }
            )));
        let jwt_key = match jwt_key.as_hmac_sha_256() {
            Ok(key) => key,
            Err(e) => return Outcome::Error(e.into()),
        };
        let claims: BTreeMap<String, String> = match jwt.verify_with_key(&jwt_key) {
            Ok(claims) => claims,
            Err(e) => {
                debug!("The provided JWT is invalid: {}", e);
                return Outcome::Error(Error::Unauthorized.into());
            }
        };
        let Some(jid) = claims.get(JWT_JID_KEY) else {
            debug!(
                "The provided JWT does not contain a '{}' claim",
                JWT_JID_KEY
            );
            return Outcome::Error(Error::Unauthorized.into());
        };
        match model::JID::try_from(jid.clone()) {
            Ok(jid) => Outcome::Success(Self(jid)),
            Err(e) => {
                debug!(
                    "The JID present in the JWT could not be parsed to a valid JID: {}",
                    e
                );
                Outcome::Error(Error::Unauthorized.into())
            }
        }
    }
}
