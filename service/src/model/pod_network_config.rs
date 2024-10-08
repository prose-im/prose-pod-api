// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::domain::Name as DomainName;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    ops::Deref,
    str::FromStr,
};

use crate::services::network_checker::IpVersion;

use super::{
    dns::{DnsEntry, DnsRecordWithStringRepr, DnsSetupStep},
    xmpp::XmppConnectionType,
    JidDomain, PodAddress,
};

pub struct PodNetworkConfig {
    pub server_domain: JidDomain,
    pub pod_address: PodAddress,
}

impl PodNetworkConfig {
    fn dns_entries(&self) -> Vec<DnsSetupStep<DnsEntry>> {
        let Self {
            server_domain,
            pod_address,
        } = self;

        // === Helpers to create DNS records ===

        // NOTE: Because of how data is modeled, sometimes we are sure this will never fail.
        fn domain_name_unchecked(str: &str) -> DomainName {
            DomainName::from_str(str).expect(&format!("Invalid domain name: {str}"))
        }
        let a = |ipv4: Ipv4Addr| DnsEntry::Ipv4 {
            hostname: domain_name_unchecked(&format!("xmpp.{server_domain}")),
            ipv4,
        };
        let aaaa = |ipv6: Ipv6Addr| DnsEntry::Ipv6 {
            hostname: domain_name_unchecked(&format!("xmpp.{server_domain}")),
            ipv6,
        };
        let srv_c2s = |target: String| DnsEntry::SrvC2S {
            hostname: domain_name_unchecked(&server_domain),
            target: domain_name_unchecked(&target),
        };
        let srv_s2s = |target: String| DnsEntry::SrvS2S {
            hostname: domain_name_unchecked(&server_domain),
            target: domain_name_unchecked(&target),
        };

        // === Helpers to create DNS setup steps ===

        let step_static_ip = |ipv4: &Option<Ipv4Addr>, ipv6: &Option<Ipv6Addr>| DnsSetupStep {
            purpose: "specify your server IP address".to_string(),
            records: vec![
                ipv4.to_owned().map(a),
                ipv6.to_owned().map(aaaa),
            ]
            .into_iter()
            .flatten()
            .collect(),
        };
        let step_c2s = |target: String| DnsSetupStep {
            purpose: "let clients connect to your server".to_string(),
            records: vec![srv_c2s(target)],
        };
        let step_s2s = |target: String| DnsSetupStep {
            purpose: "let servers connect to your server".to_string(),
            records: vec![srv_s2s(target)],
        };

        // === Main logic ===

        match pod_address {
            PodAddress::Static { ipv4, ipv6 } => vec![
                step_static_ip(ipv4, ipv6),
                step_c2s(format!("xmpp.{server_domain}.")),
                step_s2s(format!("xmpp.{server_domain}.")),
            ],
            PodAddress::Dynamic { hostname } => vec![
                step_c2s(format!("{hostname}.")),
                step_s2s(format!("{hostname}.")),
            ],
        }
    }

    pub fn dns_setup_steps(&self) -> impl Iterator<Item = DnsSetupStep<DnsRecordWithStringRepr>> {
        self.dns_entries().into_iter().map(|step| DnsSetupStep {
            purpose: step.purpose,
            records: step
                .records
                .into_iter()
                .map(DnsEntry::into_dns_record)
                .map(DnsRecordWithStringRepr::from)
                .collect(),
        })
    }

    pub fn dns_checks(&self) -> impl Iterator<Item = DnsCheck> + Clone {
        self.dns_entries()
            .into_iter()
            .flat_map(|step| step.records)
            .map(DnsCheck::from)
    }

    pub fn port_reachabilty_checks(&self) -> Vec<PortReachabilityCheck> {
        let Self { server_domain, .. } = self;
        // NOTE: Because of how data is modeled, sometimes we are sure this will never fail.
        let server_domain = &DomainName::from_str(server_domain)
            .expect(&format!("Invalid domain name: {server_domain}"));

        vec![
            PortReachabilityCheck::Xmpp {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::C2S,
            },
            PortReachabilityCheck::Xmpp {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
            },
            PortReachabilityCheck::Https {
                hostname: server_domain.clone(),
            },
        ]
    }

    pub fn ip_connectivity_checks(&self) -> Vec<IpConnectivityCheck> {
        let Self { server_domain, .. } = self;
        // NOTE: Because of how data is modeled, sometimes we are sure this will never fail.
        let server_domain = &DomainName::from_str(server_domain)
            .expect(&format!("Invalid domain name: {server_domain}"));

        vec![
            IpConnectivityCheck::XmppServer {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V4,
            },
            IpConnectivityCheck::XmppServer {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V6,
            },
            IpConnectivityCheck::XmppServer {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
            },
            IpConnectivityCheck::XmppServer {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct DnsCheck {
    pub dns_entry: DnsEntry,
}

impl Deref for DnsCheck {
    type Target = DnsEntry;

    fn deref(&self) -> &Self::Target {
        &self.dns_entry
    }
}

impl From<DnsEntry> for DnsCheck {
    fn from(dns_entry: DnsEntry) -> Self {
        Self { dns_entry }
    }
}

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
    pub fn description(&self) -> String {
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
