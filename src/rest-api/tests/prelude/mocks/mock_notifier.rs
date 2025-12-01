// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::notifications::notifier::{GenericNotifier, NotificationTrait, NotifierError};

use super::prelude::*;

#[derive(Debug)]
pub struct MockNotifier<N: NotificationTrait> {
    pub(crate) state: Arc<RwLock<MockNotifierState<N>>>,
}

#[derive(Debug)]
pub struct MockNotifierState<N: NotificationTrait> {
    pub online: bool,
    pub send_count: usize,
    pub sent: Vec<N>,
}

impl<N: NotificationTrait> Default for MockNotifier<N> {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl<N: NotificationTrait> Default for MockNotifierState<N> {
    fn default() -> Self {
        Self {
            online: true,
            send_count: Default::default(),
            sent: Default::default(),
        }
    }
}

impl<N: NotificationTrait> MockNotifier<N> {
    fn check_online(&self) -> Result<(), NotifierError> {
        if self.state.read().unwrap().online {
            Ok(())
        } else {
            Err(NotifierError("XMPP server offline".to_owned()))?
        }
    }
}

impl<N: NotificationTrait + Clone> GenericNotifier for MockNotifier<N> {
    type Notification = N;

    fn name(&self) -> &'static str {
        "dummy_notifier"
    }

    fn test_connection(&self) -> Result<bool, NotifierError> {
        Ok(self.state.read().unwrap().online)
    }

    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.send_count += 1;
        state.sent.push(notification.to_owned());
        Ok(())
    }
}
