// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod data;
pub mod durations;
mod email_address;
mod serializable_secret_string;
pub mod xmpp;

pub use data::*;
pub use durations::*;
pub use email_address::*;
pub use serializable_secret_string::*;
pub use xmpp::{
    jid, BareJid, FullJid, JidDomain, JidNode, XmppConnectionType, XmppDirectionalConnectionType,
    JID,
};
