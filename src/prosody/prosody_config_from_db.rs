// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::server_config::Model as ServerConfig;

use super::ProsodyConfig;

pub fn prosody_config_from_db(model: ServerConfig) -> ProsodyConfig {
    let mut config = ProsodyConfig::default();

    if model.message_archive_enabled {
        config.modules_enabled.insert("mam".to_string());
        config.archive_expires_after = Some(model.message_archive_retention);
    }

    config
}

// Some old method definitions
// FIXME: Remove useless code below

// fn add_admin(&mut self, jid: JID) {
//     self.config.admins.insert(jid);
// }
// fn remove_admin(&mut self, jid: &JID) {
//     self.config.admins.remove(jid);
// }

// fn add_enabled_module(&mut self, module_name: String) -> bool {
//     self.config.enabled_modules.insert(module_name)
// }
// fn remove_enabled_module(&mut self, module_name: &String) -> bool {
//     self.config.enabled_modules.remove(module_name)
// }

// fn add_disabled_module(&mut self, module_name: String) -> bool {
//     self.config.disabled_modules.insert(module_name)
// }
// fn remove_disabled_module(&mut self, module_name: &String) -> bool {
//     self.config.disabled_modules.remove(module_name)
// }

// fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate) {
//     self.config
//         .limits
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .rate = Some(value)
// }
// fn set_burst_limit(&mut self, conn_type: ConnectionType, value: DurationTime) {
//     self.config
//         .limits
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .burst = Some(value)
// }
// fn set_timeout(&mut self, value: DurationTime) {
//     self.config.limits_resolution = Some(value.seconds().clone());
// }

// fn enable_message_archiving(&mut self) -> bool {
//     self.add_enabled_module("mam".to_string())
// }
// fn disable_message_archiving(&mut self) -> bool {
//     self.remove_enabled_module(&"mam".to_string())
// }
