// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod mutation;
pub mod prosody;
mod query;
pub mod server_ctl;

pub use mutation::*;
pub use prosody::{prosody_config_from_db, ProsodyCtl};
pub use query::*;
pub use server_ctl::ServerCtl;

pub use prosody_config::ProsodyConfig;
pub use sea_orm;

pub trait ProseDefault {
    fn prose_default() -> Self;
}
