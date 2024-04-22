// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[cfg(debug_assertions)]
use crate::config::NotifierDependencyMode;
use crate::config::{Config, ConfigNotify};
use crate::notifier::{EmailNotifier, Notification};

use self::live::LiveNotifier;
#[cfg(debug_assertions)]
use self::logging::LoggingNotifier;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Notifier {
    implem: Arc<dyn NotifierImpl>,
}

impl Notifier {
    pub fn new(implem: Arc<LiveNotifier>) -> Self {
        Self { implem }
    }

    #[cfg(not(debug_assertions))]
    pub fn from_config(_config: &Config) -> Self {
        Self {
            implem: Arc::new(LiveNotifier::new(Arc::new(Mutex::new(EmailNotifier)))),
        }
    }

    #[cfg(debug_assertions)]
    pub fn from_config(config: &Config) -> Self {
        Self {
            implem: match config.dependency_modes.notifier {
                NotifierDependencyMode::Live => {
                    Arc::new(LiveNotifier::new(Arc::new(Mutex::new(EmailNotifier))))
                }
                NotifierDependencyMode::Logging => Arc::new(LoggingNotifier),
            },
        }
    }

    pub fn dispatch(
        &self,
        notification: &Notification,
        config: &ConfigNotify,
    ) -> Result<(), String> {
        self.implem.dispatch(notification, config)
    }
}

trait NotifierImpl: Send + Sync + Debug {
    fn dispatch(&self, notification: &Notification, config: &ConfigNotify) -> Result<(), String>;
}

mod live {
    use super::NotifierImpl;
    use crate::config::ConfigNotify;
    use crate::notifier::Notification;
    pub(super) use crate::notifier::Notifier as LiveNotifier;

    impl NotifierImpl for LiveNotifier {
        fn dispatch(
            &self,
            notification: &Notification,
            config: &ConfigNotify,
        ) -> Result<(), String> {
            self.implem
                .lock()
                .expect("`GenericNotifier` lock poisonned")
                .dispatch(notification, config)
        }
    }
}

#[cfg(debug_assertions)]
mod logging {
    use log::debug;

    use super::NotifierImpl;
    use crate::config::ConfigNotify;
    use crate::notifier::{notification_message, notification_subject, Notification};

    #[derive(Debug)]
    pub(super) struct LoggingNotifier;

    impl NotifierImpl for LoggingNotifier {
        fn dispatch(
            &self,
            notification: &Notification,
            _config: &ConfigNotify,
        ) -> Result<(), String> {
            let subject = notification_subject(notification);
            let message = notification_message(notification);

            debug!("Sending notification '{subject}'… Message:\n{message}");

            Ok(())
        }
    }
}
