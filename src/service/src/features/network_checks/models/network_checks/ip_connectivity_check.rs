// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

use async_trait::async_trait;
use hickory_proto::rr::domain::Name as DomainName;
use tracing::instrument;

use crate::{
    models::xmpp::XmppConnectionType,
    network_checks::{util::flattened_srv_lookup, IpVersion, NetworkChecker},
};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is an `enum` so we can derive a SSE event ID from concrete values. If it was a `struct`,
///   we wouldn't be sure all cases are mapped 1:1 to a SSE event (without keeping concerns separate).
#[derive(Clone)]
pub enum IpConnectivityCheck {
    XmppServer {
        hostname: DomainName,
        conn_type: XmppConnectionType,
        ip_version: IpVersion,
    },
}

impl Debug for IpConnectivityCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}/{}", Self::check_type(), self.id()), f)
    }
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

#[derive(Debug)]
#[derive(strum::Display)]
pub enum IpConnectivityCheckId {
    #[strum(to_string = "IPv4-c2s")]
    Ipv4C2S,
    #[strum(to_string = "IPv6-c2s")]
    Ipv6C2S,
    #[strum(to_string = "IPv4-s2s")]
    Ipv4S2S,
    #[strum(to_string = "IPv6-s2s")]
    Ipv6S2S,
}

impl From<&IpConnectivityCheck> for IpConnectivityCheckId {
    fn from(check: &IpConnectivityCheck) -> Self {
        match check {
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V4,
                ..
            } => Self::Ipv4C2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V6,
                ..
            } => Self::Ipv6C2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
                ..
            } => Self::Ipv4S2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
                ..
            } => Self::Ipv6S2S,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpConnectivityCheckResult {
    Success,
    Failure,
}

impl RetryableNetworkCheckResult for IpConnectivityCheckResult {
    fn should_retry(&self) -> bool {
        matches!(self, Self::Failure)
    }
}

#[async_trait]
impl NetworkCheck for IpConnectivityCheck {
    type CheckId = IpConnectivityCheckId;
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
    fn check_type() -> &'static str {
        "ip"
    }
    fn id(&self) -> Self::CheckId {
        <Self as NetworkCheck>::CheckId::from(self)
    }
    #[instrument(name = "IpConnectivityCheck::run", level = "trace", skip_all, fields(check = format!("{self:?}")), ret)]
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        let mut status = IpConnectivityCheckResult::Failure;
        for hostname in self.hostnames().iter() {
            if flattened_srv_lookup(
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
