// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::prosody_config::ProsodyConfig;
use crate::server_ctl::*;
use ::model::JID;

#[derive(Debug)]
pub struct ProsodyCtl {
    config: ProsodyConfig,
}

impl ProsodyCtl {
    pub fn new() -> Self {
        Self {
            config: ProsodyConfig::default(),
        }
    }
}

impl ServerCtlImpl for ProsodyCtl {
    fn save_config(&self) {
        todo!()
    }

    fn add_admin(&mut self, jid: JID) {
        self.config.admins.insert(jid);
    }
    fn remove_admin(&mut self, jid: &JID) {
        self.config.admins.remove(jid);
    }

    fn add_enabled_module(&mut self, module_name: String) -> bool {
        self.config.enabled_modules.insert(module_name)
    }
    fn remove_enabled_module(&mut self, module_name: &String) -> bool {
        self.config.enabled_modules.remove(module_name)
    }

    fn add_disabled_module(&mut self, module_name: String) -> bool {
        self.config.disabled_modules.insert(module_name)
    }
    fn remove_disabled_module(&mut self, module_name: &String) -> bool {
        self.config.disabled_modules.remove(module_name)
    }

    fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate) {
        self.config
            .limits
            .entry(conn_type.into())
            .or_insert_with(Default::default)
            .rate = Some(value)
    }
    fn set_burst_limit(&mut self, conn_type: ConnectionType, value: DurationTime) {
        self.config
            .limits
            .entry(conn_type.into())
            .or_insert_with(Default::default)
            .burst = Some(value)
    }
    fn set_timeout(&mut self, value: DurationTime) {
        self.config.limits_resolution = Some(value.seconds().clone());
    }

    fn enable_message_archiving(&mut self) -> bool {
        self.add_enabled_module("mam".to_string())
    }
    fn disable_message_archiving(&mut self) -> bool {
        self.remove_enabled_module(&"mam".to_string())
    }
}
