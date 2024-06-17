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
use entity::{model::JID, server_config::Model as ServerConfig};
pub use jwt_service::{JWTError, JWTKey, JWTService, JWT_JID_KEY};
pub use mutation::*;
pub use prosody::prosody_config_from_db;
pub use query::*;
pub use server_ctl::ServerCtl;
use uuid::Uuid;
pub use xmpp::xmpp_service::{self, *};

pub use prosody_config::{ProsodyConfigFile, ProsodyConfigFileSection};
pub use sea_orm;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self;
}

// TODO: Use `Jid`s everywhere and remove this
pub(crate) fn into_jid(from: &JID) -> xmpp_parsers::Jid {
    xmpp_parsers::Jid::new(from.to_string().as_str()).unwrap()
}

// TODO: Use `FullJid`s everywhere and remove this
pub(crate) fn into_full_jid(from: &JID) -> xmpp_parsers::FullJid {
    xmpp_parsers::FullJid::new(&format!("{}/{}", from, Uuid::new_v4())).unwrap()
}

// TODO: Use `BareJid`s everywhere and remove this
pub(crate) fn into_bare_jid(from: &JID) -> xmpp_parsers::BareJid {
    xmpp_parsers::BareJid::new(from.to_string().as_str()).unwrap()
}
