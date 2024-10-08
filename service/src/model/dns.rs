// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::{
    domain::Name as DomainName,
    rdata::{self, A, AAAA},
    RData, Record as HickoryRecord, RecordType,
};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, strum::EnumDiscriminants, Eq)]
#[strum_discriminants(derive(strum::EnumString))]
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
    SRV {
        hostname: DomainName,
        ttl: u32,
        priority: u16,
        weight: u16,
        port: u16,
        target: DomainName,
    },
}

/// NOTE: Only used in tests.
#[cfg(debug_assertions)]
impl DnsRecord {
    pub fn record_type(&self) -> DnsRecordDiscriminants {
        DnsRecordDiscriminants::from(self)
    }
    pub fn hostname(&self) -> &DomainName {
        match self {
            Self::A { hostname, .. } | Self::AAAA { hostname, .. } | Self::SRV { hostname, .. } => {
                hostname
            }
        }
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
            Self::SRV {
                hostname,
                ttl,
                priority,
                weight,
                port,
                target,
            } => format!("{hostname} {ttl} IN SRV {priority} {weight} {port} {target}"),
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
            Some(RData::A(A(ipv4))) => Ok(Self::A {
                hostname: record.name().clone(),
                ttl: record.ttl(),
                value: ipv4.clone(),
            }),
            Some(RData::AAAA(AAAA(ipv6))) => Ok(Self::AAAA {
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
            _ => Err(UnsupportedDnsRecordType(record.record_type())),
        }
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
            DnsRecord::SRV { hostname, port, .. } => Self::SRV { hostname, port },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSetupStep<Record> {
    /// The purpose of this step.
    ///
    /// Example: "specify your server IP address".
    ///
    /// Note that it always starts with a lowercase letter.
    pub purpose: String,
    pub records: Vec<Record>,
}

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
    SrvS2S {
        hostname: DomainName,
        target: DomainName,
    },
}

impl DnsEntry {
    /// e.g. "IPv4 record for xmpp.crisp.chat" or "SRV record for client-to-server connections".
    pub fn description(&self) -> String {
        match self {
            Self::Ipv4 { hostname, .. } => format!("IPv4 record for {hostname}"),
            Self::Ipv6 { hostname, .. } => format!("IPv6 record for {hostname}"),
            Self::SrvC2S { .. } => {
                format!("SRV record for client-to-server connections")
            }
            Self::SrvS2S { .. } => {
                format!("SRV record for server-to-server connections")
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
            DnsEntry::SrvC2S { hostname, target } => DnsRecord::SRV {
                hostname,
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5222,
                target,
            },
            DnsEntry::SrvS2S { hostname, target } => DnsRecord::SRV {
                hostname,
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5269,
                target,
            },
        }
    }
}
