// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, time::Duration};

use email_address::EmailAddress;
use lettre::{
    address::AddressError,
    message::{Mailbox, Message},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
        Error as SmtpError, SmtpTransport,
    },
    Address, Transport,
};
use secrecy::{ExposeSecret as _, SecretString};

use crate::{
    app_config::{ConfigBranding, MissingConfiguration},
    AppConfig,
};

use super::{GenericNotifier, NotificationTrait, NotifierError, DISPATCH_TIMEOUT_SECONDS};

#[derive(Debug, Clone)]
pub struct EmailNotifier {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<SecretString>,
    smtp_encrypt: bool,
}

impl TryFrom<&AppConfig> for EmailNotifier {
    type Error = MissingConfiguration;

    fn try_from(app_config: &AppConfig) -> Result<Self, Self::Error> {
        let email_config = app_config.notify.email()?;

        Ok(Self {
            smtp_host: email_config.smtp_host.to_owned(),
            smtp_port: email_config.smtp_port,
            smtp_username: email_config.smtp_username.to_owned(),
            smtp_password: email_config.smtp_password.to_owned(),
            smtp_encrypt: email_config.smtp_encrypt,
        })
    }
}

impl GenericNotifier for EmailNotifier {
    type Notification = EmailNotification;

    fn name(&self) -> &'static str {
        "email"
    }

    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        // Build up the email
        let email_message = Message::builder()
            .to(notification.to.clone())
            .from(notification.from.clone())
            .subject(notification.subject.clone())
            .body(notification.message.clone())
            .map_err(SendError::BuildEmail)?;

        // Create the transport if not present
        let transport = acquire_transport(
            &self.smtp_host,
            self.smtp_port,
            self.smtp_username.to_owned(),
            self.smtp_password.to_owned(),
            self.smtp_encrypt,
        )
        .map_err(SendError::BuildTransport)?;

        // Deliver the message
        transport.send(&email_message).map_err(SendError::Send)?;

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

#[derive(Debug, Clone)]
pub struct EmailNotification {
    pub from: Mailbox,
    pub to: Mailbox,
    pub subject: String,
    pub message: String,
}

impl EmailNotification {
    pub fn new(
        to: EmailAddress,
        subject: String,
        message: String,
        app_config: &AppConfig,
    ) -> Result<Self, EmailNotificationCreateError> {
        let email_config = app_config.notify.email()?;

        Ok(Self {
            from: email_config.pod_mailbox(&app_config.branding),
            to: Mailbox::new(
                None,
                to.email()
                    .parse::<Address>()
                    .map_err(CreateError::ParseTo)?,
            ),
            subject,
            message,
        })
    }
}

impl crate::app_config::ConfigNotifyEmail {
    pub fn pod_address(&self) -> Address {
        self.pod_address.email().parse().expect("`pod_address` was parsed to a valid `email_address::EmailAddress` but it's invalid according to `lettre`.")
    }
    pub fn pod_mailbox(&self, branding: &ConfigBranding) -> Mailbox {
        Mailbox::new(Some(branding.page_title.clone()), self.pod_address())
    }
}

impl Display for EmailNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  From: {}", self.from)?;
        write!(f, "  To: {}", self.to)?;
        write!(f, "  Subject: {}", self.subject)?;
        write!(
            f,
            "  Message: {}",
            self.message
                .lines()
                .map(|l| format!("    {l}"))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
        Ok(())
    }
}

impl NotificationTrait for EmailNotification {}

type CreateError = EmailNotificationCreateError;

#[derive(Debug, thiserror::Error)]
pub enum EmailNotificationCreateError {
    #[error("{0}")]
    AppConfig(#[from] MissingConfiguration),
    #[error("Invalid `to` address: {0}")]
    ParseTo(AddressError),
}

type SendError = EmailNotificationSendError;

#[derive(Debug, thiserror::Error)]
pub enum EmailNotificationSendError {
    #[error("Failed to build email: {0}")]
    BuildEmail(lettre::error::Error),
    #[error("Failed to build email transport: {0}")]
    BuildTransport(lettre::transport::smtp::Error),
    #[error("Failed to send email: {0}")]
    Send(lettre::transport::smtp::Error),
}

impl From<SendError> for NotifierError {
    fn from(err: SendError) -> Self {
        Self(err.to_string())
    }
}
