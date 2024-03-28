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
use log::debug;

use super::generic::{
    notification_message, notification_subject, GenericNotifier, Notification,
    DISPATCH_TIMEOUT_SECONDS,
};
use crate::config::ConfigNotify;
use crate::APP_CONF;

pub struct EmailNotifier;

impl GenericNotifier for EmailNotifier {
    fn name(&self) -> &'static str {
        "email"
    }

    fn attempt(&self, config: &ConfigNotify, notification: &Notification) -> Result<(), String> {
        let Some(ref email_config) = config.email else {
            return Err("No email config found".to_string());
        };

        // Build up the message text
        let subject = notification_subject(&notification);
        let message = notification_message(&notification);

        debug!("Sending email notification with message: {}", &message);

        // Build up the email
        let email_message = Message::builder()
            .to(Mailbox::new(
                None,
                email_config
                    .to
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid 'to' value in email config: {e}"))?,
            ))
            .from(Mailbox::new(
                Some(APP_CONF.branding.page_title.to_owned()),
                email_config
                    .from
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid 'from' value in email config: {e}"))?,
            ))
            .subject(subject)
            .body(message)
            .map_err(|e| format!("Could not build email: {e}"))?;

        // Create the transport if not present
        let transport = match acquire_transport(
            &email_config.smtp_host,
            email_config.smtp_port,
            email_config.smtp_username.to_owned(),
            email_config.smtp_password.to_owned(),
            email_config.smtp_encrypt,
        ) {
            Ok(email_config) => email_config,
            Err(err) => {
                return Err(format!("failed to build email transport: {err}"));
            }
        };

        // Deliver the message
        if let Err(err) = transport.send(&email_message) {
            return Err(format!("failed to send email: {err}"));
        }

        Ok(())
    }
}

fn acquire_transport(
    smtp_host: &str,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_encrypt: bool,
) -> Result<SmtpTransport, SmtpError> {
    // Acquire credentials (if any)
    let credentials = if let (Some(smtp_username_value), Some(smtp_password_value)) =
        (smtp_username, smtp_password)
    {
        Some(Credentials::new(
            smtp_username_value.to_owned(),
            smtp_password_value.to_owned(),
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
