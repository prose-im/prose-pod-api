// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod atoms;
mod avatar;
mod color;
mod db;
pub mod durations;
mod email_address;
mod lua;
mod measurements;
mod paginated;
mod pagination;
pub mod sea_orm;
mod serializable_secret_string;
mod url;
pub mod xmpp;

pub use self::atoms::*;
pub use self::avatar::*;
pub use self::color::*;
pub use self::db::*;
pub use self::durations::*;
pub use self::email_address::*;
pub use self::lua::*;
pub use self::measurements::*;
pub use self::paginated::*;
pub use self::pagination::*;
pub use self::serializable_secret_string::*;
pub use self::url::*;
pub use self::xmpp::{
    jid, BareJid, FullJid, JidDomain, JidNode, XmppConnectionType, XmppDirectionalConnectionType,
    JID,
};
