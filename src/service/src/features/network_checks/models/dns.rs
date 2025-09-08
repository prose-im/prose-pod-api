// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    ops::Deref,
};

use hickory_proto::rr::{
    domain::Name as DomainName, rdata, RData, Record as HickoryRecord, RecordType,
};
use serdev::Serialize;

use crate::xmpp::{xmpp_client_domain, xmpp_server_domain};

#[derive(Debug, Clone, Eq)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumString, strum::IntoStaticStr))]
#[serde(tag = "type")]
pub enum DnsRecord {
    A {
        hostname: DomainName,
        ttl: u32,
        value: Ipv4Addr,
    },
    AAAA {
        hostname: DomainName,
        ttl: u32,
        value: Ipv6Addr,
    },
    CNAME {
        hostname: DomainName,
        ttl: u32,
        target: DomainName,
    },
    SRV {
        hostname: DomainName,
        ttl: u32,
        priority: u16,
        weight: u16,
        port: u16,
        target: DomainName,
    },
}

impl DnsRecord {
    pub fn hostname(&self) -> &DomainName {
        match self {
            Self::A { hostname, .. }
            | Self::AAAA { hostname, .. }
            | Self::CNAME { hostname, .. }
            | Self::SRV { hostname, .. } => hostname,
        }
    }
}

/// NOTE: Only used in tests.
#[cfg(debug_assertions)]
impl DnsRecord {
    pub fn record_type(&self) -> DnsRecordDiscriminants {
        DnsRecordDiscriminants::from(self)
    }
}

impl ToString for DnsRecord {
    fn to_string(&self) -> String {
        match self {
            Self::A {
                hostname,
                ttl,
                value,
            } => format!("{hostname} {ttl} IN A {value}"),
            Self::AAAA {
                hostname,
                ttl,
                value,
            } => format!("{hostname} {ttl} IN AAAA {value}"),
            Self::CNAME {
                hostname,
                ttl,
                target,
            } => format!("{hostname} {ttl} IN CNAME {target}"),
            Self::SRV {
                hostname,
                ttl,
                priority,
                weight,
                port,
                target,
            } => {
                let mut target = target.to_string();
                if target.as_bytes().last() != Some(&b'.') {
                    target.push('.');
                }
                format!("{hostname} {ttl} IN SRV {priority} {weight} {port} {target}")
            }
        }
    }
}

impl DnsRecord {
    pub fn into_hickory_record(self) -> HickoryRecord {
        match self {
            Self::A {
                hostname,
                ttl,
                value,
            } => HickoryRecord::from_rdata(hostname, ttl, RData::A(rdata::A(value))),
            Self::AAAA {
                hostname,
                ttl,
                value,
            } => HickoryRecord::from_rdata(hostname, ttl, RData::AAAA(rdata::AAAA(value))),
            Self::CNAME {
                hostname,
                ttl,
                target,
            } => HickoryRecord::from_rdata(hostname, ttl, RData::CNAME(rdata::CNAME(target))),
            Self::SRV {
                hostname,
                ttl,
                priority,
                weight,
                port,
                target,
            } => HickoryRecord::from_rdata(
                hostname,
                ttl,
                RData::SRV(rdata::SRV::new(priority, weight, port, target)),
            ),
        }
    }
}

impl Into<HickoryRecord> for DnsRecord {
    fn into(self) -> HickoryRecord {
        self.into_hickory_record()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unsupported DNS record type: {0}")]
pub struct UnsupportedDnsRecordType(RecordType);

impl TryFrom<&HickoryRecord> for DnsRecord {
    type Error = UnsupportedDnsRecordType;

    fn try_from(record: &HickoryRecord) -> Result<Self, Self::Error> {
        match record.data() {
            Some(RData::A(rdata::A(ipv4))) => Ok(Self::A {
                hostname: record.name().clone(),
                ttl: record.ttl(),
                value: ipv4.clone(),
            }),
            Some(RData::AAAA(rdata::AAAA(ipv6))) => Ok(Self::AAAA {
                hostname: record.name().clone(),
                ttl: record.ttl(),
                value: ipv6.clone(),
            }),
            Some(RData::SRV(srv)) => Ok(Self::SRV {
                hostname: record.name().clone(),
                ttl: record.ttl(),
                priority: srv.priority(),
                weight: srv.weight(),
                port: srv.port(),
                target: srv.target().clone(),
            }),
            Some(RData::CNAME(rdata::CNAME(target))) => Ok(Self::CNAME {
                hostname: record.name().clone(),
                ttl: record.ttl(),
                target: target.clone(),
            }),
            _ => Err(UnsupportedDnsRecordType(record.record_type())),
        }
    }
}
impl TryFrom<HickoryRecord> for DnsRecord {
    type Error = UnsupportedDnsRecordType;

    fn try_from(record: HickoryRecord) -> Result<Self, Self::Error> {
        Self::try_from(&record)
    }
}

impl PartialEq for DnsRecord {
    /// Delegates equality logic to hickory.
    fn eq(&self, other: &Self) -> bool {
        self.clone().into_hickory_record() == other.clone().into_hickory_record()
    }
}

impl DnsRecord {
    /// Whether a DNS record is similar to another (e.g. different SRV weight or target but same port).
    pub fn equiv(&self, other: &Self) -> bool {
        PartialDnsRecord::from(self) == PartialDnsRecord::from(other)
    }
}

/// When we check if a user's DNS configuration is correct, we still want to accept
/// DNS records that match our requirements even if they're not exactly whan we expected.
/// This type carries only the data we really care about, allowing equality testing partial DNS records.
#[derive(Debug, PartialEq)]
enum PartialDnsRecord<'a> {
    A {
        hostname: &'a DomainName,
    },
    AAAA {
        hostname: &'a DomainName,
    },
    CNAME {
        hostname: &'a DomainName,
    },
    SRV {
        hostname: &'a DomainName,
        port: &'a u16,
    },
}

impl<'a> From<&'a DnsRecord> for PartialDnsRecord<'a> {
    fn from(record: &'a DnsRecord) -> Self {
        match record {
            DnsRecord::A { hostname, .. } => Self::A { hostname },
            DnsRecord::AAAA { hostname, .. } => Self::AAAA { hostname },
            DnsRecord::CNAME { hostname, .. } => Self::CNAME { hostname },
            DnsRecord::SRV { hostname, port, .. } => Self::SRV { hostname, port },
        }
    }
}

#[derive(Debug)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct DnsRecordWithStringRepr {
    #[serde(flatten)]
    pub inner: DnsRecord,
    pub string_repr: String,
}

impl Deref for DnsRecordWithStringRepr {
    type Target = DnsRecord;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<DnsRecord> for DnsRecordWithStringRepr {
    fn from(dns_record: DnsRecord) -> Self {
        Self {
            string_repr: dns_record.to_string(),
            inner: dns_record,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct DnsSetupStep<Record> {
    /// The purpose of this step.
    ///
    /// Example: "specify your server IP address".
    ///
    /// Note that it always starts with a lowercase letter.
    pub purpose: String,
    pub records: Vec<Record>,
}

/// NOTE: This is an `enum` so we can derive a SSE ID from concrete values.
///   If it was a `struct`, we wouldn’t be sure all cases are mapped 1:1 to
///   a SSE (without keeping concerns separate).
#[derive(Debug, Clone)]
pub enum DnsEntry {
    Ipv4 {
        hostname: DomainName,
        ipv4: Ipv4Addr,
    },
    Ipv6 {
        hostname: DomainName,
        ipv6: Ipv6Addr,
    },
    SrvC2S {
        hostname: DomainName,
        target: DomainName,
    },
    CnameWebApp {
        hostname: DomainName,
        target: DomainName,
    },
    CnameDashboard {
        hostname: DomainName,
        target: DomainName,
    },
    SrvS2S {
        hostname: DomainName,
        target: DomainName,
    },
    SrvS2SGroups {
        hostname: DomainName,
        target: DomainName,
    },
}

impl DnsEntry {
    /// e.g. "IPv4 record for prose.crisp.chat" or "SRV record for client-to-server connections".
    pub fn description(&self) -> String {
        match self {
            Self::Ipv4 { hostname, .. } => format!("IPv4 record for {hostname}"),
            Self::Ipv6 { hostname, .. } => format!("IPv6 record for {hostname}"),
            Self::SrvC2S { .. } => {
                format!("SRV record for client-to-server connections")
            }
            Self::CnameWebApp { .. } => {
                format!("CNAME record for Prose’s Web app")
            }
            Self::CnameDashboard { .. } => {
                format!("CNAME record for Prose’s Dashboard")
            }
            Self::SrvS2S { .. } => {
                format!("SRV record for server-to-server connections")
            }
            Self::SrvS2SGroups { .. } => {
                format!("SRV record for external connections to groups")
            }
        }
    }

    pub fn into_dns_record(self) -> DnsRecord {
        match self {
            DnsEntry::Ipv4 { hostname, ipv4 } => DnsRecord::A {
                hostname,
                ttl: 600,
                value: ipv4,
            },
            DnsEntry::Ipv6 { hostname, ipv6 } => DnsRecord::AAAA {
                hostname,
                ttl: 600,
                value: ipv6,
            },
            DnsEntry::CnameWebApp { hostname, target } => DnsRecord::CNAME {
                hostname,
                ttl: 600,
                target,
            },
            DnsEntry::CnameDashboard { hostname, target } => DnsRecord::CNAME {
                hostname,
                ttl: 600,
                target,
            },
            DnsEntry::SrvC2S { hostname, target } => DnsRecord::SRV {
                hostname: xmpp_client_domain(&hostname),
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5222,
                target,
            },
            DnsEntry::SrvS2S { hostname, target } => DnsRecord::SRV {
                hostname: xmpp_server_domain(&hostname),
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5269,
                target,
            },
            DnsEntry::SrvS2SGroups { hostname, target } => DnsRecord::SRV {
                hostname: xmpp_server_domain(&hostname),
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5269,
                target,
            },
        }
    }
}
