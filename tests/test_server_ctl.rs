// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::prosody::ProsodyCtl;
use prose_pod_api::server_ctl::ServerCtlImpl;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TestServerCtl {
    prosody_ctl: ProsodyCtl,
    state: Arc<Mutex<TestServerCtlState>>,
}

#[derive(Default, Debug)]
pub struct TestServerCtlState {
    pub conf_reload_count: usize,
}

impl TestServerCtl {
    pub fn new(state: Arc<Mutex<TestServerCtlState>>) -> Self {
        Self {
            prosody_ctl: ProsodyCtl::new(),
            state,
        }
    }
}

impl ServerCtlImpl for TestServerCtl {
    fn start(&self) {
        self.prosody_ctl.start()
    }

    fn stop(&self) {
        self.prosody_ctl.stop()
    }

    fn restart(&self) {
        self.prosody_ctl.restart()
    }

    fn reload(&self) {
        self.state.lock().unwrap().conf_reload_count += 1;
        self.prosody_ctl.reload()
    }

    fn status(&self) {
        self.prosody_ctl.status()
    }

    fn add_user(&self) {
        self.prosody_ctl.add_user()
    }

    fn set_user_password(&self) {
        self.prosody_ctl.set_user_password()
    }

    fn remove_user(&self) {
        self.prosody_ctl.remove_user()
    }
}
