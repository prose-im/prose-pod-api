// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, time::Duration};

use lettre::{
    address::AddressError,
    message::{Mailbox, Message, MultiPart, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
        Error as SmtpError, SmtpTransport,
    },
    Address, Transport,
};
use secrecy::{ExposeSecret as _, SecretString};

use crate::{
    app_config::{BrandingConfig, MissingConfiguration},
    models::EmailAddress,
    AppConfig,
};

use super::{GenericNotifier, NotificationTrait, NotifierError, DISPATCH_TIMEOUT_SECONDS};

#[derive(Debug, Clone)]
pub struct EmailNotifier {
    smtp_transport: SmtpTransport,
}

impl TryFrom<&AppConfig> for EmailNotifier {
    type Error = EmailNotifierCreateError;

    fn try_from(app_config: &AppConfig) -> Result<Self, Self::Error> {
        let email_config = app_config.notify.email()?;

        let smtp_transport = acquire_transport(
            &email_config.smtp_host,
            email_config.smtp_port,
            email_config.smtp_username.to_owned(),
            email_config.smtp_password.to_owned(),
            email_config.smtp_encrypt,
        )?;

        Ok(Self { smtp_transport })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmailNotifierCreateError {
    #[error("{0}")]
    AppConfig(#[from] MissingConfiguration),
    #[error("Failed to build email transport: {0}")]
    BuildTransport(#[from] lettre::transport::smtp::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to test SMTP connection: {0}")]
pub struct EmailNotifierTestConnectionError(lettre::transport::smtp::Error);

impl From<EmailNotifierTestConnectionError> for NotifierError {
    fn from(err: EmailNotifierTestConnectionError) -> Self {
        Self(err.to_string())
    }
}

impl GenericNotifier for EmailNotifier {
    type Notification = EmailNotification;

    fn name(&self) -> &'static str {
        "email"
    }

    fn test_connection(&self) -> Result<bool, NotifierError> {
        self.smtp_transport
            .test_connection()
            .map_err(EmailNotifierTestConnectionError)
            .map_err(NotifierError::from)
    }

    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        // Build up the email
        let email_message = Message::builder()
            .to(notification.to.clone())
            .from(notification.from.clone())
            .subject(notification.subject.clone())
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(notification.message_plain.clone()))
                    .singlepart(SinglePart::html(notification.message_html.clone())),
            )
            .map_err(SendError::BuildEmail)?;

        // Deliver the message
        self.smtp_transport
            .send(&email_message)
            .map_err(SendError::Send)?;

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
    pub message_plain: String,
    pub message_html: String,
}

impl EmailNotification {
    pub fn new(
        to: EmailAddress,
        subject: String,
        message_plain: String,
        message_html: String,
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
            message_plain,
            message_html,
        })
    }
}

impl crate::app_config::NotifyEmailConfig {
    pub fn pod_address(&self) -> Address {
        self.pod_address.email().parse().expect("`pod_address` was parsed to a valid `email_address::EmailAddress` but it's invalid according to `lettre`.")
    }
    pub fn pod_mailbox(&self, branding: &BrandingConfig) -> Mailbox {
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
            self.message_plain
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
    #[error("Failed to send email: {0}")]
    Send(lettre::transport::smtp::Error),
}

impl From<SendError> for NotifierError {
    fn from(err: SendError) -> Self {
        Self(err.to_string())
    }
}
