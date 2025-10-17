// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serdev::{Deserialize, Serialize};
use service::{
    auth::{
        auth_controller, errors::InvalidCredentials, AuthService, AuthToken, PasswordResetToken,
        UserInfo,
    },
    members::MemberRole,
    models::{EmailAddress, SerializableSecretString},
    notifications::NotificationService,
    util::either::Either,
    xmpp::{BareJid, XmppServiceContext},
};

use crate::{
    error::{self, Error},
    AppState,
};

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

// MARK: - Recovery email address

pub async fn set_member_recovery_email_address_route(
    State(AppState {
        ref identity_provider,
        ..
    }): State<AppState>,
    Path(jid): Path<BareJid>,
    ref ctx: XmppServiceContext,
    ref caller: UserInfo,
    Json(email_address): Json<EmailAddress>,
) -> Result<(), Error> {
    if !(caller.jid == jid || caller.is_admin()) {
        Err(error::Forbidden("You cannot do that.".to_string()))?
    }

    identity_provider
        .set_recovery_email_address(&jid, email_address, ctx)
        .await?;

    Ok(())
}

pub async fn get_member_recovery_email_address_route(
    State(AppState {
        ref identity_provider,
        ..
    }): State<AppState>,
    ref caller: UserInfo,
    ref ctx: XmppServiceContext,
    Path(jid): Path<BareJid>,
) -> Result<Json<Option<EmailAddress>>, Error> {
    if !(caller.jid == jid || caller.is_admin()) {
        Err(error::Forbidden("You cannot do that.".to_string()))?
    }

    let email_address = identity_provider
        .get_recovery_email_address_with_fallback(&jid, ctx)
        .await?;

    Ok(Json(email_address))
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "test", derive(Serialize))]
pub struct ResetPasswordRequest {
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
