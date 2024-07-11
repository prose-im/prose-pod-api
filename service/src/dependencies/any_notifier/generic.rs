// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::notification::NotificationPayload;
use tracing::{debug, info};

use crate::config::ConfigBranding;

use std::fmt::Debug;

pub(super) const DISPATCH_TIMEOUT_SECONDS: u64 = 10;

pub type Notification = NotificationPayload;

pub trait GenericNotifier: Debug + Sync + Send {
    fn name(&self) -> &'static str;
    fn attempt(&self, branding: &ConfigBranding, notification: &Notification)
        -> Result<(), String>;

    fn dispatch(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        info!(
            "Dispatching '{}' notification via '{}'…",
            notification.template(),
            self.name(),
        );

        // Attempt notification dispatch
        self.attempt(branding, notification).map_err(|e| {
            format!(
                "Failed dispatching '{}' notification via '{}': {e}",
                notification.template(),
                self.name(),
            )
        })?;

        debug!(
            "Dispatched '{}' notification via '{}'",
            notification.template(),
            self.name(),
        );
        Ok(())
    }
}

pub fn notification_subject(branding: &ConfigBranding, notification: &Notification) -> String {
    match notification {
        Notification::WorkspaceInvitation { .. } => {
            format!(
                "You have been invited to {}'s Prose server!",
                branding.company_name
            )
        }
    }
}

pub fn notification_message(branding: &ConfigBranding, notification: &Notification) -> String {
    match notification {
        Notification::WorkspaceInvitation {
            accept_link,
            reject_link,
        } => {
            vec![
                format!(
                    "You have been invited to {}'s Prose server!",
                    branding.company_name
                )
                .as_str(),
                format!(
                    "To join, open the following link in a web browser: {}. You will be guided to create an account.",
                    accept_link
                )
                .as_str(),
                // TODO: Make this "three days" dynamic
                "This link is valid for three days. After that time passes, you will have to ask a workspace anministrator to invite you again.",
                "See you soon 👋",
                format!(
                    "If you have been invited by mistake, you can reject the invitation using the following link: {}. Your email address will be erased from {}'s {} database.",
                    reject_link,
                    branding.company_name,
                    branding.page_title,
                ).as_str(),
            ]
            .join("\n\n")
        }
    }
}
