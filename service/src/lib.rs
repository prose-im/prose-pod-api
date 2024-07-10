// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub extern crate prose_xmpp;
pub extern crate xmpp_parsers;

pub mod config;
pub mod controllers;
pub mod dependencies;
pub mod model;
mod mutation_error;
pub mod prosody;
pub mod repositories;
pub mod services;
pub mod util;

use config::Config;
pub use dependencies::any_notifier as notifier;
pub use entity::model::{EmailAddress, JIDNode, MemberRole};
use entity::server_config::Model as ServerConfig;
pub use mutation_error::*;
pub use prosody::prosody_config_from_db;
pub use reqwest::Client as HttpClient;

pub use prosody_config::ProsodyConfigSection;
pub use sea_orm;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self;
}
