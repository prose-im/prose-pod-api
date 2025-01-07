// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::instrument;

use super::notifier::{email::EmailNotification, Notifier, NotifierError};

#[derive(Debug, Clone)]
pub struct NotificationService {
    email_notifier: Notifier<EmailNotification>,
}

impl NotificationService {
    pub fn new(email_notifier: Notifier<EmailNotification>) -> Self {
        Self { email_notifier }
    }

    #[instrument(level = "trace", skip_all, err)]
    pub fn send_email(&self, notification: EmailNotification) -> Result<(), self::Error> {
        self.email_notifier.dispatch(&notification)?;
        Ok(())
    }
}

pub type Error = NotificationServiceError;

#[derive(Debug, thiserror::Error)]
pub enum NotificationServiceError {
    #[error("Could not dispatch notification: {0}")]
    CouldNotDispatch(#[from] NotifierError),
}
