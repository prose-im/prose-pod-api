// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    Json,
};
use service::{
    auth::{
        auth_controller, auth_service::AuthToken, errors::InvalidCredentials, AuthService, UserInfo,
    },
    members::{MemberRole, MemberService},
    models::SerializableSecretString,
    util::either::Either,
    xmpp::BareJid,
};

use crate::{error::Error, AppState};

use super::guards::BasicAuth;

// MARK: LOG IN

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct LoginToken(SerializableSecretString);

#[derive(serde::Serialize, serde::Deserialize)]
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
        Err(Either::E1(err @ InvalidCredentials)) => Err(Error::from(err)),
        Err(Either::E2(err)) => Err(Error::from(err)),
    }
}

// MARK: ROLES

pub async fn set_member_role_route(
    State(AppState { ref db, .. }): State<AppState>,
    ref member_service: MemberService,
    ref user_info: UserInfo,
    Path(jid): Path<BareJid>,
    Json(role): Json<MemberRole>,
) -> Result<Json<MemberRole>, Error> {
    match auth_controller::set_member_role(db, member_service, user_info, jid, role).await? {
        () => Ok(Json(role)),
    }
}

// MARK: BOILERPLATE

impl From<AuthToken> for LoginToken {
    fn from(value: AuthToken) -> Self {
        Self(SerializableSecretString::from(value.0))
    }
}
