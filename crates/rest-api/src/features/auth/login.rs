// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use service::auth::{auth_service::AuthToken, AuthService};

use crate::{error::Error, guards::LazyGuard, models::SerializableSecretString};

use super::guards::BasicAuth;

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct LoginToken(SerializableSecretString);

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: LoginToken,
}

/// Log user in and return an authentication token.
#[rocket::post("/v1/login")]
pub async fn login_route(
    basic_auth: LazyGuard<BasicAuth>,
    auth_service: &State<AuthService>,
) -> Result<rocket::serde::json::Json<LoginResponse>, Error> {
    let basic_auth = basic_auth.inner?;

    let token = auth_service
        .log_in(&basic_auth.jid, &basic_auth.password)
        .await?;

    let response = LoginResponse {
        token: LoginToken::from(token),
    };
    Ok(response.into())
}

pub async fn login_route_axum(
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
