// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{models::xmpp::BareJid, wrapper_type};

wrapper_type!(EmailAddress, email_address::EmailAddress [+FromStr]; serde_with::DeserializeFromStr);

impl From<&BareJid> for EmailAddress {
    fn from(jid: &BareJid) -> Self {
        Self(email_address::EmailAddress::new_unchecked(jid.as_str()))
    }
}
