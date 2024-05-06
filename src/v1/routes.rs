// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::State;
use serde::{Deserialize, Serialize};
use service::ServerCtl;

use crate::guards::{BasicAuth, JWTService};
use crate::v1::R;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Log user in and return an authentication token.
#[post("/v1/login")]
pub(super) fn login(
    basic_auth: BasicAuth,
    jwt_service: &State<JWTService>,
    server_ctl: &State<ServerCtl>,
) -> R<LoginResponse> {
    server_ctl
        .implem
        .lock()
        .expect("Serverctl lock poisonned")
        .test_user_password(&basic_auth.jid, &basic_auth.password)?;

    let token = jwt_service.generate_jwt(&basic_auth.jid)?;

    let response = LoginResponse { token }.into();

    Ok(response)
}
