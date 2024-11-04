// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{serde::json::Json, State};
use secrecy::{ExposeSecret as _, Secret, SecretString, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};
use service::services::auth_service::AuthService;

use crate::{error::Error, guards::LazyGuard};

use super::guards::BasicAuth;

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct LoginToken(String);

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: Secret<LoginToken>,
}

/// Log user in and return an authentication token.
#[post("/v1/login")]
pub async fn login_route(
    basic_auth: LazyGuard<BasicAuth>,
    auth_service: &State<AuthService>,
) -> Result<Json<LoginResponse>, Error> {
    let basic_auth = basic_auth.inner?;
    let token = auth_service
        .log_in(&basic_auth.jid, &basic_auth.password)
        .await?;
    let response = LoginResponse {
        token: LoginToken::from(token).into(),
    }
    .into();
    Ok(response)
}

// BOILERPLATE

impl Zeroize for LoginToken {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl SerializableSecret for LoginToken {}
impl From<SecretString> for LoginToken {
    fn from(value: SecretString) -> Self {
        Self(value.expose_secret().to_owned())
    }
}
impl Into<SecretString> for LoginToken {
    fn into(self) -> SecretString {
        SecretString::new(self.0)
    }
}
