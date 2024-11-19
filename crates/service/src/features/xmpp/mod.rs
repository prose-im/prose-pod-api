// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod live_server_ctl;
pub mod live_xmpp_service;
pub mod server_ctl;
pub mod server_manager;
pub mod xmpp_service;
pub mod models {
    pub use crate::models::xmpp::*;
}

pub use live_server_ctl::*;
pub use live_xmpp_service::*;
pub use models::*;
pub use server_ctl::{ServerCtl, ServerCtlError, ServerCtlImpl};
pub use server_manager::{CreateServiceAccountError, ServerManager, ServerManagerError};
pub use xmpp_service::{
    VCard, XmppService, XmppServiceContext, XmppServiceError, XmppServiceImpl, XmppServiceInner,
};
