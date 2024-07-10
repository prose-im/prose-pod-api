// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::jid::NodePart;

use crate::{model::ServerConfig, prose_xmpp::BareJid, xmpp_parsers::jid};

pub fn to_bare_jid(jid: &entity::model::JID) -> Result<BareJid, jid::Error> {
    BareJid::new(jid.to_string().as_str())
}

pub fn bare_jid_from_username(
    username: &str,
    server_config: &ServerConfig,
) -> Result<BareJid, String> {
    Ok(BareJid::from_parts(
        Some(&NodePart::new(username).map_err(|err| format!("Invalid username: {err}"))?),
        &server_config.domain(),
    ))
}
