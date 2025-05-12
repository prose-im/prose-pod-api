// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    future,
};

use async_trait::async_trait;
use hickory_proto::rr::domain::Name as DomainName;
use tracing::instrument;

use crate::{
    models::XmppConnectionType,
    network_checks::{util::flattened_srv_lookup, NetworkChecker},
};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is an `enum` so we can derive a SSE event ID from concrete values. If it was a `struct`,
///   we wouldn't be sure all cases are mapped 1:1 to a SSE event (without keeping concerns separate).
#[derive(Clone)]
pub enum PortReachabilityCheck {
    Xmpp {
        hostname: DomainName,
        conn_type: XmppConnectionType,
    },
    Https {
        hostname: DomainName,
    },
}

impl Debug for PortReachabilityCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}/{}", Self::check_type(), self.id()), f)
    }
}

impl PortReachabilityCheck {
    pub fn port(&self) -> u16 {
        match self {
            Self::Xmpp { conn_type, .. } => conn_type.standard_port(),
            Self::Https { .. } => 443,
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

#[derive(Debug)]
#[derive(strum::Display)]
pub enum PortReachabilityCheckId {
    #[strum(to_string = "TCP-c2s")]
    TcpC2S,
    #[strum(to_string = "TCP-s2s")]
    TcpS2S,
    #[strum(to_string = "TCP-HTTPS")]
    TcpHttps,
}

impl From<&PortReachabilityCheck> for PortReachabilityCheckId {
    fn from(check: &PortReachabilityCheck) -> Self {
        match check {
            PortReachabilityCheck::Xmpp {
                conn_type: XmppConnectionType::C2S,
                ..
            } => Self::TcpC2S,
            PortReachabilityCheck::Xmpp {
                conn_type: XmppConnectionType::S2S,
                ..
            } => Self::TcpS2S,
            PortReachabilityCheck::Https { .. } => Self::TcpHttps,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortReachabilityCheckResult {
    Open,
    Closed,
}

impl RetryableNetworkCheckResult for PortReachabilityCheckResult {
    fn is_failure(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

#[async_trait]
impl NetworkCheck for PortReachabilityCheck {
    type CheckId = PortReachabilityCheckId;
    type CheckResult = PortReachabilityCheckResult;

    fn description(&self) -> String {
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
    fn check_type() -> &'static str {
        "port"
    }
    fn id(&self) -> Self::CheckId {
        <Self as NetworkCheck>::CheckId::from(self)
    }
    #[instrument(name = "PortReachabilityCheck::run", level = "trace", skip_all, fields(check = format!("{self:?}")), ret)]
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        let mut status = PortReachabilityCheckResult::Closed;
        for hostname in self.hostnames().iter() {
            if flattened_srv_lookup(
                &hostname.to_string(),
                |host| future::ready(network_checker.is_port_open(host, self.port())),
                network_checker,
            )
            .await
            {
                status = PortReachabilityCheckResult::Open;
                break;
            }
        }
        status
    }
}
