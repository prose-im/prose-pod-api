// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, RwLock};

use service::notifications::notifier::{GenericNotifier, NotificationTrait, NotifierError};

#[derive(Debug, Clone)]
pub struct MockNotifier<N: NotificationTrait + Clone> {
    pub(crate) state: Arc<RwLock<MockNotifierState<N>>>,
}

#[derive(Debug, Clone)]
pub struct MockNotifierState<N: NotificationTrait> {
    pub send_count: usize,
    pub sent: Vec<N>,
}

impl<N: NotificationTrait + Clone> Default for MockNotifier<N> {
    fn default() -> Self {
        Self {
            state: Arc::new(Default::default()),
        }
    }
}

impl<N: NotificationTrait + Clone> Default for MockNotifierState<N> {
    fn default() -> Self {
        Self {
            send_count: Default::default(),
            sent: Default::default(),
        }
    }
}

impl<N: NotificationTrait + Clone> GenericNotifier for MockNotifier<N> {
    type Notification = N;

    fn name(&self) -> &'static str {
        "dummy_notifier"
    }

    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        let mut state = self.state.write().unwrap();
        state.send_count += 1;
        state.sent.push(notification.to_owned());
        Ok(())
    }
}
