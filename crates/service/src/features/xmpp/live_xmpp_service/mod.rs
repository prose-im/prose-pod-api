// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod live_xmpp_service;
mod non_standard_xmpp_client;
mod xmpp_client;

pub use live_xmpp_service::LiveXmppService;
pub(crate) use non_standard_xmpp_client::NonStandardXmppClient;
