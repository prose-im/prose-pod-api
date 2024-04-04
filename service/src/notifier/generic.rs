// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::notification::NotificationContent;
use log::{debug, info};

use crate::{config::ConfigNotify, APP_CONF};

pub(super) const DISPATCH_TIMEOUT_SECONDS: u64 = 10;

pub type Notification = NotificationContent;

pub trait GenericNotifier: Sync + Send {
    fn name(&self) -> &'static str;
    fn attempt(&self, config: &ConfigNotify, notification: &Notification) -> Result<(), String>;

    fn dispatch(&self, notification: &Notification, config: &ConfigNotify) -> Result<(), String> {
        info!(
            "Dispatching '{}' notification via '{}'…",
            notification.template(),
            self.name(),
        );

        // Attempt notification dispatch
        self.attempt(config, notification).map_err(|e| {
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

pub fn notification_subject(notification: &Notification) -> String {
    match notification {
        Notification::MemberInvite { .. } => {
            format!(
                "You have been invited to {}'s Prose server!",
                APP_CONF.branding.company_name
            )
        }
    }
}

pub fn notification_message(notification: &Notification) -> String {
    match notification {
        Notification::MemberInvite {
            accept_link,
            reject_link,
        } => {
            vec![
                format!(
                    "You have been invited to {}'s Prose server!",
                    APP_CONF.branding.company_name
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
                    APP_CONF.branding.company_name,
                    APP_CONF.branding.page_title,
                ).as_str(),
            ]
            .join("\n\n")
        }
    }
}