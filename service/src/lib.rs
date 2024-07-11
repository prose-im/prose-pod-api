// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub extern crate prose_xmpp;
pub extern crate xmpp_parsers;

mod auth_service;
pub mod config;
pub mod dependencies;
mod jwt_service;
mod mutation;
pub mod notifier;
pub mod prosody;
mod query;
pub mod server_ctl;
pub mod xmpp;

pub use auth_service::{
    AuthError, AuthService, AuthServiceImpl, LiveAuthService, JWT_PROSODY_TOKEN_KEY,
};
use config::Config;
use entity::server_config::Model as ServerConfig;
pub use jwt_service::{JWTError, JWTKey, JWTService, JWT_JID_KEY};
pub use mutation::*;
pub use prosody::prosody_config_from_db;
pub use query::*;
pub use reqwest::Client as HttpClient;
pub use server_ctl::ServerCtl;
pub use xmpp::xmpp_service::{self, *};

pub use prosody_config::ProsodyConfigSection;
pub use sea_orm;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self;
}
