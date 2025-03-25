// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serde::{Deserialize, Serialize};
use service::{
    auth::{auth_service::AuthToken, AuthService},
    models::SerializableSecretString,
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

/// Log user in and return an authentication token.
pub async fn login_route(
    basic_auth: BasicAuth,
    auth_service: AuthService,
) -> Result<Json<LoginResponse>, Error> {
    let token = auth_service
        .log_in(&basic_auth.jid, &basic_auth.password)
        .await?;

    let response = LoginResponse {
        token: LoginToken::from(token),
    };
    Ok(response.into())
}

// BOILERPLATE

impl From<AuthToken> for LoginToken {
    fn from(value: AuthToken) -> Self {
        Self(SerializableSecretString::from(value.0))
    }
}
