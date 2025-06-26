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
    /// E.g. `your-company.com.`.
    pub fn server_fqdn(&self) -> DomainName {
        self.server_domain.as_fqdn()
    }
    /// E.g. `groups.your-company.com.`.
    pub fn groups_fqdn(&self) -> DomainName {
        self.groups_domain.as_fqdn()
    }
    /// E.g. `prose.your-company.com.` or `your-company.prose.net.`!
    pub fn pod_fqdn(&self) -> DomainName {
        self.pod_address.as_fqdn(&self.server_fqdn())
    }
    /// E.g. `prose.your-company.com.`.
    pub fn app_web_fqdn(&self) -> DomainName {
        (HostName::from_str("prose").unwrap())
            .append_domain(&self.server_fqdn())
            .expect("Domain name too long when adding the `prose` prefix")
    }
    /// E.g. `admin.prose.your-company.com.`.
    pub fn dashboard_fqdn(&self) -> DomainName {
        (HostName::from_str("admin").unwrap())
            .append_domain(&self.app_web_fqdn())
            .expect("Domain name too long when adding the `admin` prefix")
    }

    #[instrument(level = "trace", skip_all)]
    fn dns_entries(&self) -> Vec<DnsSetupStep<DnsEntry>> {
        // E.g. `your-company.com.`.
        let ref server_fqdn = self.server_fqdn();
        // E.g. `groups.your-company.com.`.
        let ref groups_fqdn = self.groups_fqdn();
        // E.g. `prose.your-company.com.`.
        let ref app_web_fqdn = self.app_web_fqdn();
        // E.g. `prose.your-company.com.` or `your-company.prose.net.`!
        let ref pod_fqdn = self.pod_fqdn();
        // E.g. `admin.prose.your-company.com.`.
        let ref dashboard_fqdn = self.dashboard_fqdn();

        // === Helpers to create DNS setup steps ===

        let step_static_ip = |ipv4: &Option<Ipv4Addr>, ipv6: &Option<Ipv6Addr>| {
            let mut step = DnsSetupStep {
                purpose: "specify your server IP address".to_string(),
                records: Vec::with_capacity(2),
            };
            if let Some(ipv4) = *ipv4 {
                step.records.push(DnsEntry::Ipv4 {
                    hostname: pod_fqdn.clone(),
                    ipv4,
                });
            }
            if let Some(ipv6) = *ipv6 {
                step.records.push(DnsEntry::Ipv6 {
                    hostname: pod_fqdn.clone(),
                    ipv6,
                });
            }
            step
        };
        let step_c2s = || DnsSetupStep {
            purpose: "let users connect to your server".to_string(),
            records: vec![
                DnsEntry::SrvC2S {
                    hostname: server_fqdn.clone(),
                    target: pod_fqdn.clone(),
                },
                // NOTE: Because of the way the Dashboard displays DNS records,
                //   we can’t mix entries of different types. Therefore the
                //   Web app CNAME can’t be here (though it’d make sense).
            ],
        };
        let step_cnames = || {
            let mut step = DnsSetupStep {
                purpose: "let users connect to the Prose Web app and Dashboard".to_string(),
                records: Vec::with_capacity(2),
            };
            // NOTE: If we’re Cloud-hosting a Prose instance, one needs to CNAME
            //   `prose.{domain}` to the Prose-provided domain to access their
            //   web app. Otherwise, it’s already configured externally or via
            //   A/AAAA records.
            if app_web_fqdn != pod_fqdn {
                step.records.push(DnsEntry::CnameWebApp {
                    hostname: app_web_fqdn.clone(),
                    target: pod_fqdn.clone(),
                });
            }
            step.records.push(DnsEntry::CnameDashboard {
                hostname: dashboard_fqdn.clone(),
                target: pod_fqdn.clone(),
            });
            step
        };
        let step_s2s = || DnsSetupStep {
            purpose: "let other servers connect to your server".to_string(),
            records: vec![
                DnsEntry::SrvS2S {
                    hostname: server_fqdn.clone(),
                    target: pod_fqdn.clone(),
                },
                DnsEntry::SrvS2S {
                    hostname: groups_fqdn.clone(),
                    target: pod_fqdn.clone(),
                },
            ],
        };

        // === Main logic ===

        let mut entries = Vec::with_capacity(4);

        if let NetworkAddress::Static { ipv4, ipv6 } = &self.pod_address {
            entries.push(step_static_ip(ipv4, ipv6));
        }
        entries.push(step_c2s());
        entries.push(step_cnames());
        if self.federation_enabled {
            entries.push(step_s2s());
        }

        entries
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

impl NetworkAddress {
    pub fn as_fqdn(&self, server_fqdn: &DomainName) -> DomainName {
        match self {
            NetworkAddress::Static { .. } => (HostName::from_str("prose").unwrap())
                .append_domain(server_fqdn)
                .expect("Domain name too long when adding `prose` prefix"),
            NetworkAddress::Dynamic { domain } => {
                let mut fqdn = domain.clone();
                fqdn.set_fqdn(true);
                fqdn
            }
        }
    }
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
