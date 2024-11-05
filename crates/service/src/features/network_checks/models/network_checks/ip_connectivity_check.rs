// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use hickory_proto::rr::domain::Name as DomainName;

use crate::{
    features::network_checks::{util::flattened_run, IpVersion, NetworkChecker},
    model::xmpp::XmppConnectionType,
};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is an `enum` so we can derive a SSE event ID from concrete values. If it was a `struct`,
///   we wouldn't be sure all cases are mapped 1:1 to a SSE event (without keeping concerns separate).
#[derive(Debug, Clone)]
pub enum IpConnectivityCheck {
    XmppServer {
        hostname: DomainName,
        conn_type: XmppConnectionType,
        ip_version: IpVersion,
    },
}

impl IpConnectivityCheck {
    pub fn ip_version(&self) -> IpVersion {
        match self {
            Self::XmppServer { ip_version, .. } => ip_version.clone(),
        }
    }
    pub fn hostnames(&self) -> Vec<DomainName> {
        match self {
            Self::XmppServer {
                hostname,
                conn_type,
                ..
            } => vec![
                // Check the standard domain first
                conn_type.standard_domain(hostname.clone()),
                // Then the XMPP server's domain
                hostname.clone(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpConnectivityCheckResult {
    Success,
    Failure,
    Missing,
}

impl RetryableNetworkCheckResult for IpConnectivityCheckResult {
    fn should_retry(&self) -> bool {
        matches!(self, Self::Failure)
    }
}

#[async_trait]
impl NetworkCheck for IpConnectivityCheck {
    type CheckResult = IpConnectivityCheckResult;

    fn description(&self) -> String {
        match self {
            Self::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V4,
                ..
            } => "Client-to-server connectivity over IPv4".to_owned(),
            Self::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V6,
                ..
            } => "Client-to-server connectivity over IPv6".to_owned(),
            Self::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
                ..
            } => "Server-to-server connectivity over IPv4".to_owned(),
            Self::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
                ..
            } => "Server-to-server connectivity over IPv6".to_owned(),
        }
    }
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        let mut status = IpConnectivityCheckResult::Failure;
        for hostname in self.hostnames().iter() {
            if flattened_run(
                &hostname.to_string(),
                |host| network_checker.is_ip_available(host.to_string(), self.ip_version()),
                network_checker,
            )
            .await
            {
                status = IpConnectivityCheckResult::Success;
                break;
            }
        }
        status
    }
}
