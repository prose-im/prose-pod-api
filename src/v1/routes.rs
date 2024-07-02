// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::State;
use serde::{Deserialize, Serialize};
use service::AuthService;

use crate::guards::BasicAuth;
use crate::v1::R;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Log user in and return an authentication token.
#[post("/v1/login")]
pub(super) fn login(basic_auth: BasicAuth, auth_service: &State<AuthService>) -> R<LoginResponse> {
    let token = auth_service.log_in(&basic_auth.jid, &basic_auth.password)?;
    let response = LoginResponse { token }.into();
    Ok(response)
}
