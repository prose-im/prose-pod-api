// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::services::jwt_service::{JWTService, JWT};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for JWT {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // NOTE: We only read the first "Authorization" header.
        let Some(auth) = req.headers().get("Authorization").next() else {
            return Error::from(error::Unauthorized(
                "No 'Authorization' header found".to_string(),
            ))
            .into();
        };
        let Some(token) = auth.strip_prefix("Bearer ") else {
            return Error::from(error::Unauthorized(
                "The 'Authorization' header does not start with 'Bearer '".to_string(),
            ))
            .into();
        };

        let jwt_service = try_outcome!(request_state!(req, JWTService));

        match Self::try_from(&token.to_string().into(), jwt_service) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(err) => {
                return Error::from(error::Unauthorized(format!(
                    "The provided JWT is invalid: {err}"
                )))
                .into();
            }
        }
    }
}
