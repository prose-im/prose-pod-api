// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;
use std::ops::Deref;

use entity::model;
use log::debug;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use service::{AuthService, JWT_JID_KEY};

use crate::error::{self, Error};

use super::LazyFromRequest;

pub struct JID(model::JID);

impl Deref for JID {
    type Target = model::JID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for JID {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            debug!("No 'Authorization' header found");
            return Error::Unauthorized.into();
        };
        let Some(jwt) = auth.strip_prefix("Bearer ") else {
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
        let claims: BTreeMap<String, String> = match auth_service.implem().verify(jwt) {
            Ok(claims) => claims,
            Err(e) => {
                debug!("The provided JWT is invalid: {e}");
                return Error::Unauthorized.into();
            }
        };
        let Some(jid) = claims.get(JWT_JID_KEY) else {
            debug!(
                "The provided JWT does not contain a '{}' claim",
                JWT_JID_KEY
            );
            return Error::Unauthorized.into();
        };
        match model::JID::try_from(jid.clone()) {
            Ok(jid) => Outcome::Success(Self(jid)),
            Err(e) => {
                debug!(
                    "The JID present in the JWT could not be parsed to a valid JID: {}",
                    e
                );
                Error::Unauthorized.into()
            }
        }
    }
}
