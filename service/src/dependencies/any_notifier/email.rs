// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::{Error as SmtpError, SmtpTransport};
use lettre::{Address, Transport};
use secrecy::{ExposeSecret as _, SecretString};

use super::generic::{
    notification_message, notification_subject, GenericNotifier, Notification,
    DISPATCH_TIMEOUT_SECONDS,
};
use crate::config::{Config, ConfigBranding};

#[derive(Debug)]
pub struct EmailNotifier {
    from: Mailbox,
    to: Mailbox,
    smtp_host: String,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<SecretString>,
    smtp_encrypt: bool,
}

impl EmailNotifier {
    pub fn new(config: &Config) -> Result<Self, String> {
        let Some(ref email_config) = config.notify.email else {
            return Err("No email config found".to_string());
        };

        Ok(Self {
            from: Mailbox::new(
                Some(config.branding.page_title.to_owned()),
                email_config
                    .from
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid 'from' value in email config: {e}"))?,
            ),
            to: Mailbox::new(
                None,
                email_config
                    .to
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid 'to' value in email config: {e}"))?,
            ),
            smtp_host: email_config.smtp_host.to_owned(),
            smtp_port: email_config.smtp_port,
            smtp_username: email_config.smtp_username.to_owned(),
            smtp_password: email_config.smtp_password.to_owned(),
            smtp_encrypt: email_config.smtp_encrypt,
        })
    }
}

impl GenericNotifier for EmailNotifier {
    fn name(&self) -> &'static str {
        "email"
    }

    fn attempt(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        // Build up the message text
        let subject = notification_subject(branding, notification);
        let message = notification_message(branding, notification);

        // Build up the email
        let email_message = Message::builder()
            .to(self.to.to_owned())
            .from(self.from.to_owned())
            .subject(subject)
            .body(message)
            .map_err(|e| format!("Failed to build email: {e}"))?;

        // Create the transport if not present
        let transport = acquire_transport(
            &self.smtp_host,
            self.smtp_port,
            self.smtp_username.to_owned(),
            self.smtp_password.to_owned(),
            self.smtp_encrypt,
        )
        .map_err(|e| format!("Failed to build email transport: {e}"))?;

        // Deliver the message
        transport
            .send(&email_message)
            .map_err(|e| format!("Failed to send email: {e}"))?;

        Ok(())
    }
}

fn acquire_transport(
    smtp_host: &str,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<SecretString>,
    smtp_encrypt: bool,
) -> Result<SmtpTransport, SmtpError> {
    // Acquire credentials (if any)
    let credentials = if let (Some(smtp_username_value), Some(smtp_password_value)) =
        (smtp_username, smtp_password)
    {
        Some(Credentials::new(
            smtp_username_value.to_owned(),
            smtp_password_value.expose_secret().to_string(),
        ))
    } else {
        None
    };

    // Acquire TLS wrapper (may fail)
    let tls_wrapper = match TlsParameters::new(smtp_host.into()) {
        Ok(parameters) if smtp_encrypt => Tls::Required(parameters),
        Ok(parameters) => Tls::Opportunistic(parameters),
        Err(e) => return Err(e),
    };

    // Build transport
    let mut mailer = SmtpTransport::builder_dangerous(smtp_host)
        .port(smtp_port)
        .tls(tls_wrapper)
        .timeout(Some(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS)));

    if let Some(credentials_value) = credentials {
        mailer = mailer.credentials(credentials_value);
    }

    Ok(mailer.build())
}
