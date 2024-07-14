// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod durations;
mod email_address;
pub(crate) mod invitations;
mod jid_node;
pub mod members;
pub mod server_config;

pub use durations::*;
pub use email_address::*;
pub use invitations::*;
use jid::{DomainPart, DomainRef, NodePart, NodeRef};
pub use jid_node::*;
pub use members::*;
pub use server_config::*;

use std::str::FromStr as _;

use sea_orm::sea_query;

pub use crate::entity::{
    member::Model as Member, workspace::Model as Workspace,
    workspace_invitation::Model as Invitation,
};
use crate::{sea_orm_string, wrapper_type};

// ===== JID =====

wrapper_type!(JID, jid::BareJid);

impl JID {
    pub fn new<S1: ToString, S2: ToString>(node: S1, domain: S2) -> Result<Self, jid::Error> {
        Ok(Self(jid::BareJid::from_parts(
            Some(&NodePart::from_str(node.to_string().as_str())?),
            &DomainPart::from_str(domain.to_string().as_str())?,
        )))
    }
    pub fn from_parts(node: Option<&NodeRef>, domain: &DomainRef) -> Self {
        Self(jid::BareJid::from_parts(node, domain))
    }
}

impl From<EmailAddress> for JID {
    fn from(email_address: EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and they are equivalent to a JID.
        Self::from_str(email_address.as_str()).unwrap()
    }
}

sea_orm_string!(JID);
