// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{
    models::{EmailAddress, Url},
    notifications::notifier::email::{EmailNotification, EmailNotificationCreateError},
    AppConfig,
};

use super::PasswordResetToken;

/// All the data needed to generate the content of a password reset notification.
#[derive(Debug, Clone)]
pub struct PasswordResetNotificationPayload {
    pub reset_token: PasswordResetToken,
    pub expires_at: DateTime<Utc>,
    pub dashboard_url: Url,
}

impl EmailNotification {
    pub fn for_password_reset(
        recipient: EmailAddress,
        payload: PasswordResetNotificationPayload,
        app_config: &AppConfig,
    ) -> Result<EmailNotification, EmailNotificationCreateError> {
        EmailNotification::new(
            recipient,
            subject(),
            message_plain(&payload),
            message_html(&payload),
            app_config,
        )
    }
}

fn subject() -> String {
    "Prose password reset".to_owned()
}

fn notification_message(
    PasswordResetNotificationPayload { expires_at, .. }: &PasswordResetNotificationPayload,
    pw_reset_link: impl Display,
) -> String {
    vec![
        format!(
            "To reset you password, open the following link in a web browser: {pw_reset_link}. You will be guided to create a new one.",
        ).as_str(),

        format!(
            "This link expires at {expires_at}. After that time passes, you will have to request a new password reset.",
            expires_at = expires_at.to_rfc2822(),
        ).as_str(),
    ]
    .join("\n\n")
}

fn pw_reset_link(reset_token: &PasswordResetToken, dashboard_url: &url::Url) -> url::Url {
    use secrecy::ExposeSecret;

    // NOTE: `join` erases the fragment and query.
    dashboard_url
        .join(&format!(
            "start/recover/{token}",
            token = reset_token.expose_secret(),
        ))
        .expect("Invalid accept link")
}

fn message_plain(payload: &PasswordResetNotificationPayload) -> String {
    notification_message(
        payload,
        pw_reset_link(&payload.reset_token, &payload.dashboard_url),
    )
}

fn message_html(payload: &PasswordResetNotificationPayload) -> String {
    let pw_reset_link = format!(
        r#"<a href="{url}">{url}</a>"#,
        url = pw_reset_link(&payload.reset_token, &payload.dashboard_url),
    );
    let mut body = notification_message(payload, pw_reset_link);
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
