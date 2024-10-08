// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::domain::Name as DomainName;

use crate::{model::xmpp::XmppConnectionType, services::network_checker::NetworkChecker};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is an `enum` so we can derive a SSE event ID from concrete values. If it was a `struct`,
///   we wouldn't be sure all cases are mapped 1:1 to a SSE event (without keeping concerns separate).
#[derive(Debug, Clone)]
pub enum PortReachabilityCheck {
    Xmpp {
        hostname: DomainName,
        conn_type: XmppConnectionType,
    },
    Https {
        hostname: DomainName,
    },
}

impl PortReachabilityCheck {
    pub fn port(&self) -> u32 {
        match self {
            Self::Xmpp { conn_type, .. } => conn_type.standard_port(),
            Self::Https { .. } => 443,
        }
    }
    pub fn description(&self) -> String {
        match self {
            Self::Xmpp {
                conn_type: XmppConnectionType::C2S,
                ..
            } => format!("Client-to-server port at TCP {}", self.port()),
            Self::Xmpp {
                conn_type: XmppConnectionType::S2S,
                ..
            } => format!("Server-to-server port at TCP {}", self.port()),
            Self::Https { .. } => format!("HTTP server port at TCP {}", self.port()),
        }
    }
    pub fn hostnames(&self) -> Vec<DomainName> {
        match self {
            Self::Xmpp {
                hostname,
                conn_type,
            } => vec![
                // Check the standard domain first
                conn_type.standard_domain(hostname.clone()),
                // Then the XMPP server's domain
                hostname.clone(),
            ],
            Self::Https { hostname } => vec![hostname.clone()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortReachabilityCheckResult {
    Open,
    Closed,
}

impl RetryableNetworkCheckResult for PortReachabilityCheckResult {
    fn should_retry(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

impl NetworkCheck for PortReachabilityCheck {
    type CheckResult = PortReachabilityCheckResult;

    fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        let mut status = PortReachabilityCheckResult::Closed;
        for hostname in self.hostnames().iter() {
            if network_checker.is_port_open(&hostname.to_string(), self.port()) {
                status = PortReachabilityCheckResult::Open;
                break;
            }
        }
        status
    }
}
