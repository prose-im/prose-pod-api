// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod connection_type;
pub mod jid;

pub use connection_type::*;
pub use jid::{BareJid, FullJid, JidDomain, JidNode, JID};
pub use prose_xmpp::mods::AvatarData;
