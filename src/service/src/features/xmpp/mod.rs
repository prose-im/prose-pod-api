// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod live_xmpp_service;
pub mod xmpp_service;
pub mod models {
    pub use crate::models::xmpp::*;
}

pub use live_xmpp_service::*;
pub use models::*;
pub use xmpp_service::{VCard, XmppService, XmppServiceContext, XmppServiceError, XmppServiceImpl};
