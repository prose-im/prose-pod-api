// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use entity::model::JID;
use http_auth_basic::Credentials;
use log::debug;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

use crate::error::{self, Error};

pub struct BasicAuth {
    pub jid: JID,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            debug!("No 'Authorization' header found");
            return Error::Unauthorized.into();
        };
        match Credentials::from_header(auth.to_string()) {
            Ok(creds) => match JID::from_str(&creds.user_id) {
                Ok(jid) => Outcome::Success(Self {
                    jid,
                    password: creds.password,
                }),
                Err(err) => {
                    debug!("The JID present in the 'Authorization' header could not be parsed to a valid JID: {err}");
                    Error::Unauthorized.into()
                }
            },
            Err(err) => {
                debug!(
                    "The 'Authorization' header is not a valid Basic authentication string: {err}"
                );
                Error::Unauthorized.into()
            }
        }
    }
}
