// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) mod live_xmpp_service;
pub mod stanza;
pub(crate) mod stanza_sender;
mod user_account_service;
mod util;
pub mod xmpp_service;

pub use live_xmpp_service::LiveXmppService;
pub use stanza_sender::StanzaSender;
pub use user_account_service::UserAccountService;
