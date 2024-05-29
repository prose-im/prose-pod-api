// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod config;
pub mod dependencies;
mod mutation;
pub mod notifier;
pub mod prosody;
mod query;
pub mod server_ctl;

use config::Config;
use entity::server_config::Model as ServerConfig;
pub use mutation::*;
pub use prosody::prosody_config_from_db;
pub use query::*;
pub use server_ctl::ServerCtl;

pub use prosody_config::{ProsodyConfigFile, ProsodyConfigFileSection};
pub use sea_orm;
pub use vcard_parser;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self;
}
