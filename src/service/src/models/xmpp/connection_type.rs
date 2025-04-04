// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use hickory_proto::rr::{Name, Name as DomainName};

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
        match self {
            Self::C2S => Name::from_str("_xmpp-client._tcp")
                .unwrap()
                .append_domain(&domain)
                .unwrap(),
            Self::S2S => Name::from_str("_xmpp-server._tcp")
                .unwrap()
                .append_domain(&domain)
                .unwrap(),
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
