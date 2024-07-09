// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log::debug;
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::services::jwt_service::{JWTService, JWT};

use crate::error::{self, Error};
use crate::request_state;

use super::LazyFromRequest;

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

        let jwt_service = try_outcome!(request_state!(req, JWTService));

        match Self::try_from(token, jwt_service) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(err) => {
                debug!("The provided JWT is invalid: {err}");
                return Error::Unauthorized.into();
            }
        }
    }
}
