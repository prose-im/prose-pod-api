// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody_admin_rest;
mod prosody_config_from_db;
mod prosody_ctl;

pub use prosody_admin_rest::ProsodyAdminRest;
pub use prosody_config_from_db::prosody_config_from_db;
pub use prosody_ctl::ProsodyCtl;
