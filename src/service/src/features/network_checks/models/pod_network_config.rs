// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::domain::{Name as DomainName, Name as HostName};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use tracing::instrument;

use crate::{
    app_config::{PodAddress, PodConfig},
    network_checks::{
        models::network_checks::*, DnsEntry, DnsRecordWithStringRepr, DnsSetupStep, IpVersion,
    },
    xmpp::{JidDomain, XmppConnectionType},
};

#[derive(Debug, Clone)]
pub struct PodNetworkConfig {
    pub server_domain: JidDomain,
    pub groups_domain: JidDomain,
    pub pod_address: NetworkAddress,
    pub federation_enabled: bool,
}

impl PodNetworkConfig {
    #[instrument(level = "trace", skip_all)]
    fn dns_entries(&self) -> Vec<DnsSetupStep<DnsEntry>> {
        let Self {
            server_domain,
            pod_address,
            groups_domain,
            ..
        } = self;
        let ref server_domain = server_domain.as_fqdn();
        let ref groups_domain = groups_domain.as_fqdn();

        let ref pod_fqdn = match pod_address {
            NetworkAddress::Static { .. } => (HostName::from_str("prose").unwrap())
                .append_domain(server_domain)
                .expect("Domain name too long when adding `prose` prefix"),
            NetworkAddress::Dynamic { domain } => {
                let mut fqdn = domain.clone();
                fqdn.set_fqdn(true);
                fqdn
            }
        };

        // === Helpers to create DNS setup steps ===

        let step_static_ip = |ipv4: &Option<Ipv4Addr>, ipv6: &Option<Ipv6Addr>| DnsSetupStep {
            purpose: "specify your server IP address".to_string(),
            records: vec![
                ipv4.to_owned().map(|ipv4: Ipv4Addr| DnsEntry::Ipv4 {
                    hostname: pod_fqdn.clone(),
                    ipv4,
                }),
                ipv6.to_owned().map(|ipv6: Ipv6Addr| DnsEntry::Ipv6 {
                    hostname: pod_fqdn.clone(),
                    ipv6,
                }),
            ]
            .into_iter()
            .flatten()
            .collect(),
        };
        let step_c2s = |target: &DomainName| DnsSetupStep {
            purpose: "let clients connect to your server".to_string(),
            records: vec![
                DnsEntry::SrvC2S {
                    hostname: server_domain.clone(),
                    target: target.clone(),
                },
            ],
        };
        let step_s2s = |target: &DomainName| DnsSetupStep {
            purpose: "let other servers connect to your server".to_string(),
            records: vec![
                DnsEntry::SrvS2S {
                    hostname: server_domain.clone(),
                    target: target.clone(),
                },
                DnsEntry::SrvS2S {
                    hostname: groups_domain.clone(),
                    target: target.clone(),
                },
            ],
        };

        // === Main logic ===

        match pod_address {
            NetworkAddress::Static { ipv4, ipv6 } => {
                let mut entries = Vec::with_capacity(3);
                entries.push(step_static_ip(ipv4, ipv6));
                entries.push(step_c2s(pod_fqdn));
                if self.federation_enabled {
                    entries.push(step_s2s(pod_fqdn));
                }
                entries
            }
            NetworkAddress::Dynamic { .. } => {
                let mut entries = Vec::with_capacity(2);
                entries.push(step_c2s(pod_fqdn));
                if self.federation_enabled {
                    entries.push(step_s2s(pod_fqdn));
                }
                entries
            }
        }
    }

    /// Configuration steps shown in the "DNS setup instructions" of the Prose Pod Dashboard.
    ///
    /// They are derived from the recommended DNS configuration.
    #[instrument(level = "trace", skip_all)]
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
    /// They are derived from the recommended DNS configuration.
    #[instrument(level = "trace", skip_all)]
    pub fn dns_record_checks(&self) -> impl Iterator<Item = DnsRecordCheck> + Clone {
        self.dns_entries()
            .into_iter()
            .flat_map(|step| step.records)
            .map(DnsRecordCheck::from)
    }

    /// "Ports reachability" checks listed in the "Network configuration checker" of the Prose Pod Dashboard.
    #[instrument(level = "trace", skip_all)]
    pub fn port_reachability_checks(&self) -> Vec<PortReachabilityCheck> {
        let server_domain = self.server_domain.as_fqdn();

        let mut checks = vec![
            PortReachabilityCheck::Xmpp {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::C2S,
            },
        ];
        if self.federation_enabled {
            checks.push(PortReachabilityCheck::Xmpp {
                hostname: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
            });
        }
        checks.push(PortReachabilityCheck::Https {
            hostname: server_domain.clone(),
        });

        checks
    }

    /// "IP connectivity" checks listed in the "Network configuration checker" of the Prose Pod Dashboard.
    #[instrument(level = "trace", skip_all)]
    pub fn ip_connectivity_checks(&self) -> Vec<IpConnectivityCheck> {
        let ref server_domain = self.server_domain.as_fqdn();

        let mut checks: Vec<IpConnectivityCheck> = Vec::with_capacity(4);
        checks.push(IpConnectivityCheck::XmppServer {
            server_domain: server_domain.clone(),
            conn_type: XmppConnectionType::C2S,
            ip_version: IpVersion::V4,
        });
        checks.push(IpConnectivityCheck::XmppServer {
            server_domain: server_domain.clone(),
            conn_type: XmppConnectionType::C2S,
            ip_version: IpVersion::V6,
        });
        if self.federation_enabled {
            checks.push(IpConnectivityCheck::XmppServer {
                server_domain: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
            });
            checks.push(IpConnectivityCheck::XmppServer {
                server_domain: server_domain.clone(),
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
            });
        }

        checks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAddress {
    Static {
        ipv4: Option<Ipv4Addr>,
        ipv6: Option<Ipv6Addr>,
    },
    Dynamic {
        domain: DomainName,
    },
}

impl From<PodAddress> for NetworkAddress {
    fn from(address: PodAddress) -> Self {
        match (address.domain, address.ipv4, address.ipv6) {
            (Some(domain), _, _) => Self::Dynamic { domain },
            (None, None, None) => unreachable!("`PodAddress`es shouldn’t be constructed manually"),
            (None, ipv4, ipv6) => Self::Static { ipv4, ipv6 },
        }
    }
}

impl PodConfig {
    pub fn network_address(&self) -> NetworkAddress {
        NetworkAddress::from(self.address.clone())
    }
}

impl ToString for NetworkAddress {
    fn to_string(&self) -> String {
        match self {
            Self::Dynamic { domain } => domain.to_string(),

            Self::Static {
                ipv6: Some(ipv6), ..
            } => ipv6.to_string(),

            Self::Static {
                ipv4: Some(ipv4), ..
            } => ipv4.to_string(),

            Self::Static {
                ipv4: None,
                ipv6: None,
            } => unreachable!("IPv4 or IPv6 must be defined by this point."),
        }
    }
}
