// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::debug;

use crate::app_config::ConfigBranding;

use super::{notification_message, notification_subject, GenericNotifier, Notification};

#[derive(Debug)]
pub(super) struct LoggingNotifier;

impl GenericNotifier for LoggingNotifier {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn attempt(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        let subject = notification_subject(branding, notification);
        let message = notification_message(branding, notification);

        debug!("Sending notification '{subject}'… Message:\n{message}");

        Ok(())
    }
}
