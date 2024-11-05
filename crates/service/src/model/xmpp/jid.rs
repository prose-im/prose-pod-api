// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

pub use jid::*;
use sea_orm::sea_query;

use crate::{model::EmailAddress, sea_orm_string, wrapper_type};

// ===== BARE JID =====

wrapper_type!(JID, jid::BareJid);

impl JID {
    pub fn new<S1: ToString, S2: ToString>(node: S1, domain: S2) -> Result<Self, jid::Error> {
        Ok(Self(jid::BareJid::from_parts(
            Some(&jid::NodePart::from_str(node.to_string().as_str())?),
            &jid::DomainPart::from_str(domain.to_string().as_str())?,
        )))
    }
    pub fn from_parts(node: Option<&jid::NodeRef>, domain: &jid::DomainRef) -> Self {
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

// ===== JID NODE =====

wrapper_type!(JidNode, jid::NodePart);

impl From<EmailAddress> for JidNode {
    fn from(value: EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and their local part are equivalent to a JID node part.
        Self::from_str(value.local_part()).unwrap()
    }
}

sea_orm_string!(JidNode);

// ===== JID NODE =====

wrapper_type!(JidDomain, jid::DomainPart);

sea_orm_string!(JidDomain);
