// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use tracing::instrument;

use super::notifier::{email::EmailNotification, Notifier};

#[derive(Debug, Clone)]
pub struct NotificationService {
    email_notifier: Notifier<EmailNotification>,
}

impl NotificationService {
    pub fn new(email_notifier: Notifier<EmailNotification>) -> Self {
        Self { email_notifier }
    }

    #[instrument(level = "trace", skip_all, err)]
    pub fn send_email(&self, email: EmailNotification) -> Result<(), anyhow::Error> {
        (self.email_notifier.dispatch(&email)).context("Could not dispatch notification")?;
        Ok(())
    }
}
