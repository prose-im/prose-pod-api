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
    members::MemberRole,
    models::SerializableSecretString,
    notifications::NotificationService,
    util::either::Either,
    xmpp::{BareJid, XmppServiceContext},
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
    State(AppState {
        ref user_repository,
        ..
    }): State<AppState>,
    ref user_info: UserInfo,
    ref auth: AuthToken,
    Path(jid): Path<BareJid>,
    Json(role): Json<MemberRole>,
) -> Result<Json<MemberRole>, Error> {
    match auth_controller::set_member_role(user_repository, user_info, jid, role, auth).await? {
        () => Ok(Json(role)),
    }
}

// MARK: - Password reset / change

pub async fn request_password_reset_route(
    State(AppState {
        ref app_config,
        ref identity_provider,
        ref auth_service,
        ..
    }): State<AppState>,
    ref notification_service: NotificationService,
    ref caller: UserInfo,
    ref ctx: XmppServiceContext,
    Path(ref jid): Path<BareJid>,
) -> Result<StatusCode, Error> {
    auth_controller::request_password_reset(
        notification_service,
        app_config,
        jid,
        None,
        identity_provider,
        auth_service,
        caller,
        ctx,
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
    ref auth_service: AuthService,
    Path(token): Path<PasswordResetToken>,
    Json(ResetPasswordRequest { ref password }): Json<ResetPasswordRequest>,
) -> Result<(), Error> {
    auth_controller::reset_password(token, password, auth_service).await?;
    Ok(())
}

// MARK: - Boilerplate

impl From<AuthToken> for LoginToken {
    fn from(value: AuthToken) -> Self {
        Self(SerializableSecretString::from(value.0))
    }
}
