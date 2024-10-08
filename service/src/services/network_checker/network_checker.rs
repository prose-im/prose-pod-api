// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use tracing::debug;

use crate::model::{
    dns::{DnsEntry, DnsRecord},
    xmpp::XmppConnectionType,
};

/// A service used to perform network checks (DNS resolution, ports checking…).
#[derive(Debug, Clone)]
pub struct NetworkChecker(Arc<dyn NetworkCheckerImpl>);

impl NetworkChecker {
    pub fn new(implem: Arc<dyn NetworkCheckerImpl>) -> Self {
        Self(implem)
    }
}

impl Deref for NetworkChecker {
    type Target = Arc<dyn NetworkCheckerImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait NetworkCheckerImpl: Debug + Sync + Send {
    fn ipv4_lookup(&self, host: &str) -> Result<Vec<DnsRecord>, DnsLookupError>;
    fn ipv6_lookup(&self, host: &str) -> Result<Vec<DnsRecord>, DnsLookupError>;
    fn srv_lookup(&self, host: &str) -> Result<Vec<DnsRecord>, DnsLookupError>;

    fn check_dns_entry(&self, dns_entry: DnsEntry) -> DnsRecordStatus {
        let check = |dns_lookup_result: Result<Vec<DnsRecord>, DnsLookupError>,
                     expected: &DnsRecord|
         -> DnsRecordStatus {
            // Check the given domain but also its standard equivalent (e.g. `_xmpp-client._tcp.{domain}`).
            // If we the DNS lookup fails, consider that the DNS record is `Invalid`.
            let records = match dns_lookup_result {
                Ok(records) => records,
                Err(err) => {
                    debug!("DNS lookup failed: {err}");
                    return DnsRecordStatus::Error(err);
                }
            };

            // If we find the exact DNS record (not taking the TTL into account), return `Valid`.
            // If we find a DNS record that's close enough, we consider it `PartiallyValid`.
            // Otherwise, it's `Invalid`.
            for record in records {
                if record.eq(expected) {
                    return DnsRecordStatus::Valid;
                } else if record.equiv(expected) {
                    return DnsRecordStatus::PartiallyValid {
                        expected: expected.clone(),
                        found: record,
                    };
                }
            }

            return DnsRecordStatus::Invalid;
        };

        let check_ipv4 = |expected: &DnsRecord| -> DnsRecordStatus {
            let host = expected.hostname().to_string();
            check(self.ipv4_lookup(&host), expected)
        };

        let check_ipv6 = |expected: &DnsRecord| -> DnsRecordStatus {
            let host = expected.hostname().to_string();
            check(self.ipv6_lookup(&host), expected)
        };

        let check_srv = |expected: &DnsRecord, conn_type: XmppConnectionType| -> DnsRecordStatus {
            let host = expected.hostname();
            check(
                self.srv_lookup(&conn_type.standard_domain(host.clone()).to_string())
                    .or_else(|_err| self.srv_lookup(&host.to_string())),
                expected,
            )
        };

        match dns_entry {
            DnsEntry::Ipv4 { .. } => check_ipv4(&dns_entry.into_dns_record()),
            DnsEntry::Ipv6 { .. } => check_ipv6(&dns_entry.into_dns_record()),
            DnsEntry::SrvC2S { .. } => {
                check_srv(&dns_entry.into_dns_record(), XmppConnectionType::C2S)
            }
            DnsEntry::SrvS2S { .. } => {
                check_srv(&dns_entry.into_dns_record(), XmppConnectionType::S2S)
            }
        }
    }

    fn is_port_open(&self, host: &str, port_number: u32) -> bool;

    fn is_ipv4_available(&self, host: &str) -> bool;
    fn is_ipv6_available(&self, host: &str) -> bool;
    fn is_ip_available(&self, host: &str, ip_version: IpVersion) -> bool {
        match ip_version {
            IpVersion::V4 => self.is_ipv4_available(host),
            IpVersion::V6 => self.is_ipv6_available(host),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsRecordStatus {
    Valid,
    PartiallyValid {
        expected: DnsRecord,
        found: DnsRecord,
    },
    Invalid,
    Error(DnsLookupError),
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("DNS lookup error: {0}")]
pub struct DnsLookupError(pub String);

#[derive(Debug, Clone)]
pub enum IpVersion {
    V4,
    V6,
}
