// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) mod live_xmpp_service;
pub mod stanza;
mod xmpp_client;
pub mod xmpp_service;

pub use live_xmpp_service::LiveXmppService;
pub use xmpp_client::XMPPClient;
