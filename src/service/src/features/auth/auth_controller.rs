// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use jid::BareJid;
use tracing::instrument;

use crate::{
    errors::Forbidden,
    identity_provider::IdentityProvider,
    invitations::InvitationContact,
    members::{MemberRole, UserRepository},
    notifications::{notifier::email::EmailNotification, NotificationService},
    util::{
        either::{Either, Either4},
        JidExt as _,
    },
    xmpp::XmppServiceContext,
    AppConfig,
};

use super::{
    errors::*, password_reset_notification::PasswordResetNotificationPayload, AuthService,
    AuthToken, Credentials, Password, PasswordResetToken, UserInfo,
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
    skip_all, fields(jid = %jid, role = %role, caller = %caller.jid),
)]
pub async fn set_member_role(
    user_repository: &UserRepository,
    caller: &UserInfo,
    jid: BareJid,
    role: MemberRole,
    auth: &AuthToken,
) -> Result<(), Either4<CannotChangeOwnRole, CannotAssignRole, Forbidden, anyhow::Error>> {
    if caller.jid == jid {
        return Err(Either4::E1(CannotChangeOwnRole));
    };
    if caller.primary_role < role {
        return Err(Either4::E2(CannotAssignRole));
    };

    let username = jid.expect_username();
    user_repository.set_user_role(username, &role, auth).await?;

    Ok(())
}

#[instrument(
    name = "auth_controller::request_password_reset",
    level = "trace",
    skip_all, fields(jid = jid.to_string(), caller = caller.jid.to_string()),
)]
pub async fn request_password_reset(
    notification_service: &NotificationService,
    app_config: &AppConfig,
    jid: &BareJid,
    contact: Option<InvitationContact>,
    identity_provider: &IdentityProvider,
    auth_service: &AuthService,
    caller: &UserInfo,
    ctx: &XmppServiceContext,
) -> Result<PasswordResetNotificationPayload, Either<Forbidden, anyhow::Error>> {
    let ref auth = ctx.auth_token;

    let Some(username) = jid.node() else {
        // That’s not a user account, of course you can’t do that.
        return Err(Either::E1(Forbidden("Not a user account".to_owned())));
    };

    // Authorize action (or not).
    // NOTE: One can only reset their own password or trigger a reset for
    //   someone else if they are an admin.
    if !(caller.is(jid) || caller.is_admin()) {
        return Err(Either::E1(Forbidden(
            "Not your account. Only admins can do that.".to_owned(),
        )));
    }

    let contact = match contact {
        Some(contact) => contact,
        None => {
            let email_address = identity_provider.get_recovery_email_address_with_fallback(jid, ctx).await?.expect(
                "Until we implement #342, this should have been set already (except for the #256 bug).",
            );
            InvitationContact::Email { email_address }
        }
    };

    // Create password reset token.
    let info = auth_service
        .create_password_reset_token(username, None, &contact, auth)
        .await?;

    // Send email.
    let email_address = match contact {
        InvitationContact::Email { email_address } => email_address.clone(),
    };
    let notification_payload = PasswordResetNotificationPayload::new(
        info.token,
        info.expires_at,
        app_config.dashboard_url(),
    );
    let email =
        EmailNotification::for_password_reset(email_address, &notification_payload, app_config)
            .context("Could not create email")?;
    notification_service.send_email(email)?;

    Ok(notification_payload)
}

#[instrument(name = "auth_controller::reset_password", level = "trace", skip_all)]
pub async fn reset_password(
    token: PasswordResetToken,
    password: &Password,
    auth_service: &AuthService,
) -> Result<(), Either4<PasswordValidationError, PasswordResetTokenExpired, Forbidden, anyhow::Error>>
{
    auth_service.reset_password(token, password).await
}
