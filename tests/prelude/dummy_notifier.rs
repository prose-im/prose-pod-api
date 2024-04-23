// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::service::notifier::{GenericNotifier, Notification};

use std::sync::Mutex;

#[derive(Debug)]
pub struct DummyNotifier {
    pub(crate) state: Mutex<DummyNotifierState>,
}

#[derive(Debug, Default)]
pub struct DummyNotifierState {
    pub send_count: usize,
    pub sent: Vec<Notification>,
}

impl DummyNotifier {
    pub fn new(state: Mutex<DummyNotifierState>) -> Self {
        Self { state }
    }
}

impl GenericNotifier for DummyNotifier {
    fn name(&self) -> &'static str {
        "dummy_notifier"
    }

    fn attempt(&self, notification: &service::notifier::Notification) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.send_count += 1;
        state.sent.push(notification.clone());
        Ok(())
    }
}
