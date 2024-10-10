// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) mod live_xmpp_service;
mod non_standard_xmpp_client;
mod xmpp_client;
pub mod xmpp_service;

pub use live_xmpp_service::LiveXmppService;
pub use non_standard_xmpp_client::NonStandardXmppClient;
pub use xmpp_client::XMPPClient;
