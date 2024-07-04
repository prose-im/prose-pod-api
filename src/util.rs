// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use prose_pod_core::{
    prose_xmpp::BareJid,
    repositories::ServerConfig,
    xmpp_parsers::jid::{DomainPart, NodePart},
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
