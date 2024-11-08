// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::domain::Name as DomainName;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::services::network_checker::IpVersion;

use super::{
    dns::{DnsEntry, DnsRecordWithStringRepr, DnsSetupStep},
    network_checks::*,
    xmpp::XmppConnectionType,
    JidDomain, PodAddress,
};

pub struct PodNetworkConfig {
    pub server_domain: JidDomain,
    pub pod_address: PodAddress,
}

// NOTE: Because of how data is modeled, sometimes we are sure this will never fail.
fn domain_name_unchecked(str: &str) -> DomainName {
    DomainName::from_str(str).expect(&format!("Invalid domain name: {str}"))
}

impl PodNetworkConfig {
    fn dns_entries(&self) -> Vec<DnsSetupStep<DnsEntry>> {
        let Self {
            server_domain,
            pod_address,
        } = self;

        // === Helpers to create DNS records ===

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

    /// Configuration steps shown in the "DNS setup instructions" of the Prose Pod Dashboard.
    ///
    /// They are derived from the recommended DNS configuration (from [`PodNetworkConfig::dns_entries`]).
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

    /// "DNS records" checks listed in the "Network configuration checker" of the Prose Pod Dashboard.
    ///
    /// They are derived from the recommended DNS configuration (from [`PodNetworkConfig::dns_entries`]).
    pub fn dns_record_checks(&self) -> impl Iterator<Item = DnsRecordCheck> + Clone {
        self.dns_entries()
            .into_iter()
            .flat_map(|step| step.records)
            .map(DnsRecordCheck::from)
    }

    /// "Ports reachability" checks listed in the "Network configuration checker" of the Prose Pod Dashboard.
    pub fn port_reachability_checks(&self) -> Vec<PortReachabilityCheck> {
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

    /// "IP connectivity" checks listed in the "Network configuration checker" of the Prose Pod Dashboard.
    pub fn ip_connectivity_checks(&self) -> Vec<IpConnectivityCheck> {
        let Self {
            server_domain,
            pod_address,
        } = self;
        // NOTE: Because of how data is modeled, sometimes we are sure this will never fail.
        let server_domain = &DomainName::from_str(server_domain)
            .expect(&format!("Invalid domain name: {server_domain}"));

        let hostname = match pod_address {
            PodAddress::Static { .. } => server_domain,
            PodAddress::Dynamic { hostname } => hostname,
        };

        vec![
            IpConnectivityCheck::XmppServer {
                hostname: hostname.clone(),
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V4,
            },
            IpConnectivityCheck::XmppServer {
                hostname: hostname.clone(),
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V6,
            },
            IpConnectivityCheck::XmppServer {
                hostname: hostname.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
            },
            IpConnectivityCheck::XmppServer {
                hostname: hostname.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
            },
        ]
    }
}
