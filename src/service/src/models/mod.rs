// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod avatar;
mod color;
mod data;
pub mod durations;
mod email_address;
mod lua;
mod paginated;
mod pagination;
pub mod sea_orm;
mod serializable_secret_string;
mod url;
pub mod xmpp;

pub use avatar::*;
pub use color::*;
pub use data::*;
pub use durations::*;
pub use email_address::*;
pub use lua::*;
pub use paginated::*;
pub use pagination::*;
pub use serializable_secret_string::*;
pub use url::*;
pub use xmpp::{
    jid, BareJid, FullJid, JidDomain, JidNode, XmppConnectionType, XmppDirectionalConnectionType,
    JID,
};
