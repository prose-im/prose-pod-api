// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, str::FromStr as _};

use email_address::EmailAddress;
use secrecy::ExposeSecret as _;

use crate::{
    app_config::ConfigBranding,
    invitations::InvitationToken,
    notifications::notifier::email::{EmailNotification, EmailNotificationCreateError},
    AppConfig,
};

#[derive(Clone)]
pub struct WorkspaceInvitationPayload {
    pub accept_token: InvitationToken,
    pub reject_token: InvitationToken,
}

impl EmailNotification {
    pub fn from(
        email_recipient: EmailAddress,
        invitation_payload: WorkspaceInvitationPayload,
        app_config: &AppConfig,
    ) -> Result<EmailNotification, EmailNotificationCreateError> {
        EmailNotification::new(
            email_recipient,
            notification_subject(&app_config.branding),
            notification_message(&app_config.branding, invitation_payload),
            app_config,
        )
    }
}

pub fn notification_subject(branding: &ConfigBranding) -> String {
    format!(
        "You have been invited to {company}'s Prose server!",
        company = branding.company_name,
    )
}

pub fn notification_message(
    branding: &ConfigBranding,
    WorkspaceInvitationPayload {
        accept_token,
        reject_token,
        ..
    }: WorkspaceInvitationPayload,
) -> String {
    let admin_site_root = PathBuf::from_str(&branding.page_url.to_string()).unwrap();
    let accept_link = admin_site_root
        .join(format!(
            "invitations/accept/{token}",
            token = accept_token.into_secret_string().expose_secret(),
        ))
        .display()
        .to_string();
    let reject_link = admin_site_root
        .join(format!(
            "invitations/reject/{token}",
            token = reject_token.into_secret_string().expose_secret(),
        ))
        .display()
        .to_string();

    vec![
        format!(
            "You have been invited to {company}'s Prose server!",
            company = branding.company_name,
        )
        .as_str(),
        format!(
            "To join, open the following link in a web browser: {accept_link}. You will be guided to create an account.",
        )
        .as_str(),
        // TODO: Make this "three days" dynamic
        // "This link is valid for three days. After that time passes, you will have to ask a workspace anministrator to invite you again.",
        "See you soon 👋",
        format!(
            "If you have been invited by mistake, you can reject the invitation using the following link: {reject_link}. Your email address will be erased from {company}'s {app} database.",
            company = branding.company_name,
            app = branding.page_title,
        ).as_str(),
    ]
    .join("\n\n")
}
