// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use entity::server_config::Model as ServerConfig;
use service::{
    prose_xmpp::BareJid,
    xmpp_parsers::jid::{self, DomainPart, NodePart},
};

use crate::error::Error;

pub fn bare_jid_from_username(
    username: String,
    server_config: &ServerConfig,
) -> Result<BareJid, Error> {
    Ok(BareJid::from_parts(
        Some(
            &NodePart::from_str(&username).map_err(|err| Error::BadRequest {
                reason: format!("Invalid username: {err}"),
            })?,
        ),
        &DomainPart::from_str(&server_config.domain.to_string()).map_err(|err| {
            Error::InternalServerError {
                reason: format!("Invalid domain: {err}"),
            }
        })?,
    ))
}

pub fn to_bare_jid(jid: &entity::model::JID) -> Result<BareJid, jid::Error> {
    BareJid::new(jid.to_string().as_str())
}
