// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serdev::Serialize;
use service::{
    auth::{
        auth_controller, errors::InvalidCredentials, AuthService, AuthToken, PasswordResetToken,
        UserInfo,
    },
    members::{MemberRole, MemberService},
    models::SerializableSecretString,
    notifications::NotificationService,
    util::either::Either,
    xmpp::{BareJid, ServerCtl},
};
use validator::Validate;

use crate::{error::Error, AppState};

use super::{extractors::BasicAuth, models::Password};

// MARK: - Log in

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub struct LoginToken(SerializableSecretString);

#[derive(Debug, Serialize)]
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

// MARK: - Roles

pub async fn set_member_role_route(
    State(AppState { ref db, .. }): State<AppState>,
    ref member_service: MemberService,
    ref user_info: UserInfo,
    Path(jid): Path<BareJid>,
    Json(role): Json<MemberRole>,
) -> Result<Json<MemberRole>, Error> {
    match auth_controller::set_member_role(&db.write, member_service, user_info, jid, role).await? {
        () => Ok(Json(role)),
    }
}

// MARK: - Password reset / change

pub async fn request_password_reset_route(
    State(AppState {
        ref db,
        ref app_config,
        ..
    }): State<AppState>,
    ref notification_service: NotificationService,
    ref caller: UserInfo,
    Path(ref jid): Path<BareJid>,
) -> Result<StatusCode, Error> {
    auth_controller::request_password_reset(
        &db.write,
        notification_service,
        app_config,
        caller,
        jid,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

#[derive(Debug, Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(Serialize))]
pub struct ResetPasswordRequest {
    #[validate(nested)]
    pub password: Password,
}

pub async fn reset_password_route(
    State(AppState { ref db, .. }): State<AppState>,
    ref server_ctl: ServerCtl,
    Path(ref token): Path<PasswordResetToken>,
    Json(ResetPasswordRequest { password }): Json<ResetPasswordRequest>,
) -> Result<(), Error> {
    auth_controller::reset_password(&db.write, server_ctl, token, &password).await?;
    Ok(())
}

// MARK: - Boilerplate

impl From<AuthToken> for LoginToken {
    fn from(value: AuthToken) -> Self {
        Self(SerializableSecretString::from(value.0))
    }
}
