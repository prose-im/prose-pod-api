// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use model::JID;
use prose_pod_api::prosody_ctl::ProsodyCtl;
use prose_pod_api::server_ctl::{ConnectionType, DataRate, DurationTime, ServerCtlImpl};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TestServerCtl {
    prosody_ctl: ProsodyCtl,
    state: Arc<Mutex<TestServerCtlState>>
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
    fn save_config(&self) {
        self.state.lock().unwrap().conf_reload_count += 1;
        self.prosody_ctl.save_config()
    }

    fn add_admin(&mut self, jid: JID) {
        self.prosody_ctl.add_admin(jid)
    }
    fn remove_admin(&mut self, jid: &JID) {
        self.prosody_ctl.remove_admin(jid)
    }

    fn add_enabled_module(&mut self, module_name: String) -> bool {
        self.prosody_ctl.add_enabled_module(module_name)
    }
    fn remove_enabled_module(&mut self, module_name: &String) -> bool {
        self.prosody_ctl.remove_enabled_module(module_name)
    }

    fn add_disabled_module(&mut self, module_name: String) -> bool {
        self.prosody_ctl.add_disabled_module(module_name)
    }
    fn remove_disabled_module(&mut self, module_name: &String) -> bool {
        self.prosody_ctl.remove_disabled_module(module_name)
    }

    fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate) {
        self.prosody_ctl.set_rate_limit(conn_type, value)
    }
    fn set_burst_limit(&mut self, conn_type: ConnectionType, value: DurationTime) {
        self.prosody_ctl.set_burst_limit(conn_type, value)
    }
    fn set_timeout(&mut self, value: DurationTime) {
        self.prosody_ctl.set_timeout(value)
    }

    fn enable_message_archiving(&mut self) -> bool {
        self.prosody_ctl.enable_message_archiving()
    }
    fn disable_message_archiving(&mut self) -> bool {
        self.prosody_ctl.disable_message_archiving()
    }
}
