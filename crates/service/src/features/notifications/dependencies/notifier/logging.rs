// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::marker::PhantomData;

use tracing::info;

use super::{GenericNotifier, NotificationTrait, NotifierError};

#[derive(Debug)]
pub(super) struct LoggingNotifier<N: NotificationTrait> {
    phantom: PhantomData<N>,
}

impl<N: NotificationTrait> Default for LoggingNotifier<N> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<N: NotificationTrait> GenericNotifier for LoggingNotifier<N> {
    type Notification = N;

    fn name(&self) -> &'static str {
        "logging"
    }

    fn test_connection(&self) -> Result<bool, NotifierError> {
        Ok(true)
    }

    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        let str = notification.to_string();
        let len = str.len();
        let lines = str.lines();
        let mut str = String::with_capacity(len + lines.clone().count() * 3 - 1);
        for line in lines {
            str.push_str("  ");
            str.push_str(line);
            str.push('\n');
        }
        info!("Sending notification…\n{str}");
        Ok(())
    }
}

impl NotificationTrait for String {}
