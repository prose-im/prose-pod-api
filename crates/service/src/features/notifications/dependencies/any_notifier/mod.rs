// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod email;
mod generic;

use std::ops::Deref;

use crate::features::app_config::ConfigBranding;

pub use self::email::EmailNotifier;
pub use self::generic::{
    notification_message, notification_subject, GenericNotifier, Notification,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct AnyNotifier {
    implem: Box<dyn GenericNotifier>,
}

impl AnyNotifier {
    pub fn new(implem: Box<dyn GenericNotifier>) -> Self {
        Self { implem }
    }
}

impl Deref for AnyNotifier {
    type Target = Box<dyn GenericNotifier>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

impl GenericNotifier for AnyNotifier {
    fn name(&self) -> &'static str {
        self.deref().name()
    }

    fn attempt(
        &self,
        branding: &ConfigBranding,
        notification: &Notification,
    ) -> Result<(), String> {
        self.deref().attempt(branding, notification)
    }
}
