// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod email;
mod generic;

use std::sync::{Arc, Mutex};

use crate::config::ConfigBranding;

pub use self::email::EmailNotifier;
pub use self::generic::{
    notification_message, notification_subject, GenericNotifier, Notification,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct AnyNotifier {
    implem: Arc<Mutex<dyn GenericNotifier>>,
}

impl AnyNotifier {
    pub fn new(implem: Arc<Mutex<dyn GenericNotifier>>) -> Self {
        Self { implem }
    }
}

impl GenericNotifier for AnyNotifier {
    fn name(&self) -> &'static str {
        self.implem
            .lock()
            .expect("`GenericNotifier` lock poisonned")
            .name()
    }

    fn attempt(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        self.implem
            .lock()
            .expect("`GenericNotifier` lock poisonned")
            .attempt(branding, notification)
    }
}
