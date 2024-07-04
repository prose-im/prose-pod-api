// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{prose_xmpp::BareJid, xmpp_parsers::jid};

pub fn to_bare_jid(jid: &entity::model::JID) -> Result<BareJid, jid::Error> {
    BareJid::new(jid.to_string().as_str())
}
