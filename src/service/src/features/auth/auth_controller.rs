// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Context};
use chrono::Utc;
use jid::BareJid;
use sea_orm::DatabaseConnection;
use secrecy::SecretString;
use tracing::instrument;

use crate::{
    members::{entities::member, MemberRepository, MemberRole, MemberService},
    notifications::{notifier::email::EmailNotification, NotificationService},
    util::either::{Either, Either3},
    xmpp::ServerCtl,
    AppConfig,
};

use super::{
    errors::*, password_reset_notification::PasswordResetNotificationPayload,
    password_reset_tokens, AuthService, AuthToken, Credentials, PasswordResetRecord,
    PasswordResetToken, UserInfo,
};

/// Log user in and return an authentication token.
#[instrument(
    name = "auth_controller::log_in",
    level = "trace",
    skip_all, fields(jid = credentials.jid.to_string()),
)]
pub async fn log_in(
    credentials: &Credentials,
    auth_service: &AuthService,
) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
    auth_service
        .log_in(&credentials.jid, &credentials.password)
        .await
}

#[instrument(
    name = "auth_controller::set_member_role",
    level = "trace",
    skip_all, fields(jid = jid.to_string(), role = role.to_string(), caller = user_info.jid.to_string()),
)]
pub async fn set_member_role(
    db: &DatabaseConnection,
    member_service: &MemberService,
    user_info: &UserInfo,
    jid: BareJid,
    role: MemberRole,
) -> Result<(), Either3<CannotChangeOwnRole, CannotAssignRole, anyhow::Error>> {
    let Some(caller) = MemberRepository::get(db, &user_info.jid).await? else {
        return Err(Either3::E3(anyhow!("Cannot get role for '{jid}'.")));
    };
    if caller.jid() == jid {
        return Err(Either3::E1(CannotChangeOwnRole));
    };
    if caller.role < role {
        return Err(Either3::E2(CannotAssignRole));
    };

    member_service.set_member_role(&jid, role).await?;

    Ok(())
}

#[instrument(
    name = "auth_controller::request_password_reset",
    level = "trace",
    skip_all, fields(jid = jid.to_string(), caller = caller.jid.to_string()),
)]
pub async fn request_password_reset(
    db: &DatabaseConnection,
    notification_service: &NotificationService,
    app_config: &AppConfig,
    caller: &UserInfo,
    jid: &BareJid,
) -> Result<(), Either3<MissingEmailAddress, CannotResetPassword, anyhow::Error>> {
    // Find member email address.
    let email_address = match MemberRepository::get(db, jid).await? {
        Some(member::Model {
            email_address: Some(email_address),
            ..
        }) => email_address,
        _ => return Err(Either3::E1(MissingEmailAddress(jid.clone()))),
    };

    // Authorize action (or not).
    // NOTE: One can only reset their own password or trigger a reset for
    //   someone else if they are an admin.
    if !(caller.is(jid) || caller.is_admin()) {
        return Err(Either3::E2(CannotResetPassword));
    }

    // Generate a random token.
    // NOTE: We could generate a stronger token but it’s not valid for long and
    //   needs to be URL-safe so a UUIDv4 is fine.
    let token = PasswordResetToken::from(uuid::Uuid::new_v4().to_string());

    // Compute token expiry.
    let password_reset_token_ttl = (app_config.auth.password_reset_token_ttl.to_std()).expect(
        "`app_config.auth.password_reset_token_ttl` contains years or months. Not supported.",
    );
    let expires_at = chrono::Utc::now() + password_reset_token_ttl;

    // Store password reset token.
    let record = PasswordResetRecord {
        jid: jid.to_owned(),
        expires_at,
    };
    password_reset_tokens::set(db, &token, &record).await?;

    // Send email.
    let notification_payload = PasswordResetNotificationPayload {
        reset_token: token,
        expires_at,
        dashboard_url: app_config.dashboard_url().clone(),
    };
    let email =
        EmailNotification::for_password_reset(email_address, notification_payload, app_config)
            .context("Could not create email")?;
    notification_service.send_email(email)?;

    Ok(())
}

#[instrument(name = "auth_controller::reset_password", level = "trace", skip_all)]
pub async fn reset_password(
    db: &DatabaseConnection,
    server_ctl: &ServerCtl,
    token: &PasswordResetToken,
    password: &SecretString,
) -> Result<(), Either3<PasswordResetTokenNotFound, PasswordResetTokenExpired, anyhow::Error>> {
    let record = match password_reset_tokens::get(db, token).await? {
        Some(model) => model,
        None => return Err(Either3::E1(PasswordResetTokenNotFound)),
    };
    let jid = record.jid;

    tracing::Span::current().record("jid", jid.to_string());

    // Check password expiry.
    if Utc::now() > record.expires_at {
        return Err(Either3::E2(PasswordResetTokenExpired));
    }

    (server_ctl.set_user_password(&jid, password).await).context("ServerCtl error")?;

    // Delete record from database.
    password_reset_tokens::delete(db, token).await?;

    Ok(())
}
