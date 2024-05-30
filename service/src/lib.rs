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
pub mod xmpp;

use config::Config;
use entity::{model::JID, server_config::Model as ServerConfig};
pub use mutation::*;
pub use prosody::prosody_config_from_db;
pub use query::*;
pub use server_ctl::ServerCtl;
pub use xmpp::stanza::vcard::VCard4 as VCard;
pub use xmpp::xmpp_service::{self, *};

pub use prosody_config::{ProsodyConfigFile, ProsodyConfigFileSection};
pub use sea_orm;

trait ProseDefault {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self;
}

pub(crate) fn into_jid(from: &JID) -> xmpp_parsers::Jid {
    xmpp_parsers::Jid::new(from.to_string().as_str()).unwrap()
}
