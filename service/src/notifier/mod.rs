// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod generic;

mod email;

use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub use self::email::EmailNotifier;
pub use self::generic::{
    notification_message, notification_subject, GenericNotifier, Notification,
};

#[derive(Debug)]
pub struct Notifier {
    pub implem: Arc<Mutex<dyn GenericNotifier>>,
}

impl Notifier {
    pub fn new(implem: Arc<Mutex<dyn GenericNotifier>>) -> Self {
        Self { implem }
    }
}

impl Deref for Notifier {
    type Target = Arc<Mutex<dyn GenericNotifier>>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
