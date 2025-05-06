// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serde::{Deserialize, Serialize};
use service::{
    auth::{auth_controller, auth_service::AuthToken, errors::InvalidCredentials, AuthService},
    models::SerializableSecretString,
    util::Either,
};

use crate::error::Error;

use super::guards::BasicAuth;

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct LoginToken(SerializableSecretString);

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: LoginToken,
}

pub async fn login_route(
    basic_auth: BasicAuth,
    auth_service: AuthService,
) -> Result<Json<LoginResponse>, Error> {
    match auth_controller::log_in(&basic_auth.into(), &auth_service).await {
        Ok(token) => Ok(Json(LoginResponse {
            token: LoginToken::from(token),
        })),
        Err(Either::Left(err @ InvalidCredentials)) => Err(Error::from(err)),
        Err(Either::Right(err)) => Err(Error::from(err)),
    }
}

// BOILERPLATE

impl From<AuthToken> for LoginToken {
    fn from(value: AuthToken) -> Self {
        Self(SerializableSecretString::from(value.0))
    }
}
