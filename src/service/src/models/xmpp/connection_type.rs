// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::domain::{Name as DomainName, Name as HostName};
use lazy_static::lazy_static;

lazy_static! {
    static ref XMPP_CLIENT_DOMAIN_PREFIX: HostName =
        HostName::from_ascii("_xmpp-client._tcp").unwrap();
    static ref XMPP_SERVER_DOMAIN_PREFIX: HostName =
        HostName::from_ascii("_xmpp-server._tcp").unwrap();
}

pub(crate) fn xmpp_client_domain(domain: &DomainName) -> HostName {
    let prefix = XMPP_CLIENT_DOMAIN_PREFIX.clone();
    prefix.append_domain(domain).unwrap()
}
pub(crate) fn xmpp_server_domain(domain: &DomainName) -> HostName {
    let prefix = XMPP_SERVER_DOMAIN_PREFIX.clone();
    prefix.append_domain(domain).unwrap()
}

#[derive(Debug, Clone)]
pub enum XmppConnectionType {
    C2S,
    S2S,
}

impl XmppConnectionType {
    pub fn standard_port(&self) -> u16 {
        match self {
            Self::C2S => 5222,
            Self::S2S => 5269,
        }
    }
    pub fn standard_domain(&self, domain: DomainName) -> DomainName {
        // TODO: Refactor all of the network checks mess and get rid of this hack. See https://github.com/prose-im/prose-pod-api/issues/274.
        if domain.to_string().contains("._tcp.") {
            return domain;
        }
        match self {
            Self::C2S => xmpp_client_domain(&domain),
            Self::S2S => xmpp_server_domain(&domain),
        }
    }
}

/// Values from <https://prosody.im/doc/modules/mod_limits>.
/// Probably abstract enough to be used in non-Prosody APIs.
///
/// See also <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
#[derive(Debug, Eq, PartialEq)]
pub enum XmppDirectionalConnectionType {
    /// "c2s"
    ClientToServer,
    /// "s2sin"
    ServerToServerInbounds,
    /// "s2sout"
    ServerToServerOutbounds,
}
