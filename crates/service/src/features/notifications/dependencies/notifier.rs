// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;
use std::sync::Arc;

#[cfg(debug_assertions)]
use crate::app_config::NotifierDependencyMode;
use crate::app_config::{AppConfig, ConfigBranding};

use super::any_notifier::{EmailNotifier, Notification};

use self::live::LiveNotifier;
#[cfg(debug_assertions)]
use self::logging::LoggingNotifier;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Notifier {
    implem: Arc<dyn NotifierImpl>,
}

impl Notifier {
    #[cfg(not(debug_assertions))]
    pub fn from_config(config: &AppConfig) -> Result<Self, String> {
        Ok(LiveNotifier::new(Box::new(EmailNotifier::new(config)?)).into())
    }

    #[cfg(debug_assertions)]
    pub fn from_config(config: &AppConfig) -> Result<Self, String> {
        Ok(match config.debug_only.dependency_modes.notifier {
            NotifierDependencyMode::Live => {
                LiveNotifier::new(Box::new(EmailNotifier::new(config)?)).into()
            }
            NotifierDependencyMode::Logging => Self {
                implem: Arc::new(LoggingNotifier),
            },
        })
    }

    pub fn dispatch(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        self.implem.dispatch(branding, notification)
    }
}

impl From<LiveNotifier> for Notifier {
    fn from(value: LiveNotifier) -> Self {
        Self {
            implem: Arc::new(value),
        }
    }
}

trait NotifierImpl: Send + Sync + Debug {
    fn dispatch(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String>;
}

mod live {
    use super::NotifierImpl;
    pub(super) use crate::notifications::dependencies::any_notifier::AnyNotifier as LiveNotifier;
    use crate::{
        app_config::ConfigBranding,
        notifications::dependencies::any_notifier::{GenericNotifier, Notification},
    };

    impl NotifierImpl for LiveNotifier {
        fn dispatch(
            &self,
            branding: &ConfigBranding,
            notification: &Notification,
        ) -> Result<(), String> {
            (self as &dyn GenericNotifier).dispatch(branding, notification)
        }
    }
}

#[cfg(debug_assertions)]
mod logging {
    use tracing::debug;

    use super::NotifierImpl;
    use crate::{
        app_config::ConfigBranding,
        notifications::dependencies::any_notifier::{
            notification_message, notification_subject, Notification,
        },
    };

    #[derive(Debug)]
    pub(super) struct LoggingNotifier;

    impl NotifierImpl for LoggingNotifier {
        fn dispatch(
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
}
