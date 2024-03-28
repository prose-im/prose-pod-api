// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod config;
mod mutation;
pub mod notifier;
pub mod prosody;
mod query;
pub mod server_ctl;

pub use mutation::*;
pub use prosody::{prosody_config_from_db, ProsodyCtl};
pub use query::*;
pub use server_ctl::ServerCtl;

use lazy_static::lazy_static;
pub use prosody_config::ProsodyConfigFile;
pub use sea_orm;

use crate::config::Config;

lazy_static! {
    pub static ref APP_CONF: Config = Config::figment();
}

pub trait ProseDefault {
    fn prose_default() -> Self;
}
