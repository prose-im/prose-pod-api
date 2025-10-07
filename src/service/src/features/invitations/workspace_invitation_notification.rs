// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

use crate::{
    invitations::InvitationToken,
    models::{EmailAddress, Url},
    notifications::notifier::email::{EmailNotification, EmailNotificationCreateError},
    AppConfig,
};

/// All the data needed to generate the content of a workspace invitation.
#[derive(Debug, Clone)]
pub struct WorkspaceInvitationPayload {
    pub accept_token: InvitationToken,
    pub reject_token: InvitationToken,
    pub workspace_name: String,
    pub dashboard_url: Url,
    pub api_app_name: String,
    pub organization_name: Option<String>,
}

impl EmailNotification {
    pub fn for_workspace_invitation(
        recipient: EmailAddress,
        payload: WorkspaceInvitationPayload,
        app_config: &AppConfig,
    ) -> Result<EmailNotification, EmailNotificationCreateError> {
        EmailNotification::new(
            recipient,
            notification_subject(&payload),
            notification_message_plain(&payload),
            notification_message_html(&payload),
            app_config,
        )
    }
}

fn notification_subject(
    WorkspaceInvitationPayload {
        workspace_name,
        organization_name,
        ..
    }: &WorkspaceInvitationPayload,
) -> String {
    if let Some(ref company) = organization_name {
        if company.ends_with("s") {
            format!("You have been invited to {company}’ Prose server!")
        } else {
            format!("You have been invited to {company}’s Prose server!")
        }
    } else {
        format!("You have been invited to {workspace_name}!")
    }
}

fn notification_message(
    WorkspaceInvitationPayload {
        workspace_name,
        api_app_name,
        organization_name,
        ..
    }: &WorkspaceInvitationPayload,
    accept_link: impl Display,
    reject_link: impl Display,
) -> String {
    vec![
        if let Some(ref company) = organization_name {
            if company.ends_with("s") {
                format!("You have been invited to {company}’ Prose server!")
            } else {
                format!("You have been invited to {company}’s Prose server!")
            }
        } else {
            format!("You have been invited to {workspace_name}!")
        }
        .as_str(),
        format!(
            "To join, open the following link in a web browser: {accept_link}. You will be guided to create an account.",
        )
        .as_str(),
        // TODO: Make this "three days" dynamic
        // "This link is valid for three days. After that time passes, you will have to ask a workspace anministrator to invite you again.",
        "See you soon 👋",
        format!(
            "If you have been invited by mistake, you can reject the invitation using the following link: {reject_link}. Your email address will be erased from {database}.",
            database = if let Some(ref company) = organization_name {
                format!("{company}’s {api_app_name} database")
            } else {
                format!("the server’s database")
            }
        )
        .as_str(),
    ]
    .join("\n\n")
}

fn accept_link(accept_token: &InvitationToken, dashboard_url: &url::Url) -> url::Url {
    use secrecy::ExposeSecret as _;

    // NOTE: `join` erases the fragment and query.
    dashboard_url
        .join(&format!(
            "invitations/accept/{token}",
            token = accept_token.expose_secret(),
        ))
        .expect("Invalid accept link")
}
fn reject_link(reject_token: &InvitationToken, dashboard_url: &url::Url) -> url::Url {
    use secrecy::ExposeSecret as _;

    // NOTE: `join` erases the fragment and query.
    dashboard_url
        .join(&format!(
            "invitations/reject/{token}",
            token = reject_token.expose_secret(),
        ))
        .expect("Invalid reject link")
}

fn notification_message_plain(payload: &WorkspaceInvitationPayload) -> String {
    notification_message(
        payload,
        accept_link(&payload.accept_token, &payload.dashboard_url),
        reject_link(&payload.reject_token, &payload.dashboard_url),
    )
}

fn notification_message_html(payload: &WorkspaceInvitationPayload) -> String {
    let accept_link = format!(
        r#"<a href="{url}">{url}</a>"#,
        url = accept_link(&payload.accept_token, &payload.dashboard_url),
    );
    let reject_link = format!(
        r#"<a href="{url}">{url}</a>"#,
        url = reject_link(&payload.reject_token, &payload.dashboard_url),
    );
    let mut body = notification_message(payload, accept_link, reject_link);
    body = body.replace("\n\n", "</p><p>");
    format!(
        r#"<html lang="en">
  <head>
    <meta charset="utf-8">
    <style>
body {{
  color: #222;
  background: #fff;
  font: 100% system-ui;
}}
a {{
  color: #0033cc;
}}

@media (prefers-color-scheme: dark) {{
  body {{
    color: #eee;
    background: #121212;
  }}
  a {{
    color: #809fff;
  }}
}}
</style>
  </head>
  <body><p>{body}</p></body>
</html>"#
    )
}
