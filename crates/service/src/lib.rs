// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub extern crate prose_xmpp;
pub extern crate prosody_config;
pub extern crate reqwest;
extern crate xmpp_parsers;

pub mod dependencies;
pub mod errors;
mod features;
pub mod models;
mod mutation_error;
pub mod util;

pub use features::app_config::AppConfig;
pub use features::prosody;
use features::server_config::ServerConfig;
pub use features::*;
pub use mutation_error::MutationError;
pub use prosody::prosody_config_from_db;
pub use prosody_config::ProsodyConfigSection;
pub use reqwest::Client as HttpClient;
pub use sea_orm;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &AppConfig) -> Self;
}
