// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

pub use jid::*;

use crate::{models::EmailAddress, sea_orm_string, wrapper_type};

// MARK: - Bare JID

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

// MARK: - JID node

wrapper_type!(JidNode, jid::NodePart);

impl From<&EmailAddress> for JidNode {
    fn from(email_address: &EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and their local part are
        //   equivalent to a JID node part.
        Self::from_str(email_address.local_part()).unwrap()
    }
}

impl From<EmailAddress> for JidNode {
    fn from(email_address: EmailAddress) -> Self {
        Self::from(&email_address)
    }
}

impl From<&NodeRef> for JidNode {
    fn from(value: &NodeRef) -> Self {
        Self::from(value.to_owned())
    }
}

sea_orm_string!(JidNode);

// MARK: - JID domain

wrapper_type!(JidDomain, jid::DomainPart);

impl From<&hickory_proto::rr::Name> for JidDomain {
    #[inline]
    fn from(value: &hickory_proto::rr::Name) -> Self {
        // NOTE: Domain names are already parsed, and
        //   they are equivalent to a JID domain part.
        Self::from_str(value.to_string().as_str()).unwrap()
    }
}
impl JidDomain {
    pub fn as_fqdn(&self) -> hickory_proto::rr::Name {
        // NOTE: JID domain parts are already parsed,
        //   and they are equivalent to a domain name.
        let mut name = hickory_proto::rr::Name::from_str(self.to_string().as_str()).unwrap();
        name.set_fqdn(true);
        name
    }
}

sea_orm_string!(JidDomain);
