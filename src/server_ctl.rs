// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub struct ServerCtl {
    pub implem: Arc<Mutex<dyn ServerCtlImpl>>,
}

impl ServerCtl {
    pub fn new(implem: Arc<Mutex<dyn ServerCtlImpl>>) -> Self {
        Self { implem }
    }
}

impl Deref for ServerCtl {
    type Target = Arc<Mutex<dyn ServerCtlImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

/// Abstraction over ProsodyCtl in case we want to switch to another server.
/// Also facilitates testing.
pub trait ServerCtlImpl: Sync + Send {
    fn start(&self);
    fn stop(&self);
    fn restart(&self);
    fn reload(&self);
    fn status(&self);

    fn add_user(&self);
    fn set_user_password(&self);
    fn remove_user(&self);
}
