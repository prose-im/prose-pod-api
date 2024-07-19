// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::service::notifier::{GenericNotifier, Notification};
use service::config::ConfigBranding;

use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone)]
pub struct MockNotifier {
    pub(crate) state: Arc<RwLock<MockNotifierState>>,
}

#[derive(Debug, Clone, Default)]
pub struct MockNotifierState {
    pub send_count: usize,
    pub sent: Vec<Notification>,
}

impl MockNotifier {
    pub fn new(state: Arc<RwLock<MockNotifierState>>) -> Self {
        Self { state }
    }
}

impl GenericNotifier for MockNotifier {
    fn name(&self) -> &'static str {
        "dummy_notifier"
    }

    fn attempt(
        &self,
        _branding: &ConfigBranding,
        notification: &service::notifier::Notification,
    ) -> Result<(), String> {
        let mut state = self.state.write().unwrap();
        state.send_count += 1;
        state.sent.push(notification.clone());
        Ok(())
    }
}
