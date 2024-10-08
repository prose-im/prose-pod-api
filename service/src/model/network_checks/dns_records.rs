// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use tracing::debug;

use crate::{
    model::{
        dns::{DnsEntry, DnsRecord},
        xmpp::XmppConnectionType,
    },
    services::network_checker::{DnsLookupError, NetworkChecker},
};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is a `struct` because [`DnsCheck`] is an enum (see [`DnsCheck`]'s doc).
#[derive(Debug, Clone)]
pub struct DnsRecordCheck {
    pub dns_entry: DnsEntry,
}

impl Deref for DnsRecordCheck {
    type Target = DnsEntry;

    fn deref(&self) -> &Self::Target {
        &self.dns_entry
    }
}

impl From<DnsEntry> for DnsRecordCheck {
    fn from(dns_entry: DnsEntry) -> Self {
        Self { dns_entry }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsRecordCheckResult {
    Valid,
    PartiallyValid {
        expected: DnsRecord,
        found: DnsRecord,
    },
    Invalid,
    Error(DnsLookupError),
}

impl RetryableNetworkCheckResult for DnsRecordCheckResult {
    fn should_retry(&self) -> bool {
        matches!(self, Self::Invalid | Self::Error(_))
    }
}

impl NetworkCheck for DnsRecordCheck {
    type CheckResult = DnsRecordCheckResult;

    fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        network_checker.check_dns_entry(self.dns_entry.clone())
    }
}

impl NetworkChecker {
    fn check_dns_entry(&self, dns_entry: DnsEntry) -> DnsRecordCheckResult {
        let check = |dns_lookup_result: Result<Vec<DnsRecord>, DnsLookupError>,
                     expected: &DnsRecord|
         -> DnsRecordCheckResult {
            // Check the given domain but also its standard equivalent (e.g. `_xmpp-client._tcp.{domain}`).
            // If we the DNS lookup fails, consider that the DNS record is `Invalid`.
            let records = match dns_lookup_result {
                Ok(records) => records,
                Err(err) => {
                    debug!("DNS lookup failed: {err}");
                    return DnsRecordCheckResult::Error(err);
                }
            };

            // If we find the exact DNS record (not taking the TTL into account), return `Valid`.
            // If we find a DNS record that's close enough, we consider it `PartiallyValid`.
            // Otherwise, it's `Invalid`.
            for record in records {
                if record.eq(expected) {
                    return DnsRecordCheckResult::Valid;
                } else if record.equiv(expected) {
                    return DnsRecordCheckResult::PartiallyValid {
                        expected: expected.clone(),
                        found: record,
                    };
                }
            }

            return DnsRecordCheckResult::Invalid;
        };

        let check_ipv4 = |expected: &DnsRecord| -> DnsRecordCheckResult {
            let host = expected.hostname().to_string();
            check(self.ipv4_lookup(&host), expected)
        };

        let check_ipv6 = |expected: &DnsRecord| -> DnsRecordCheckResult {
            let host = expected.hostname().to_string();
            check(self.ipv6_lookup(&host), expected)
        };

        let check_srv =
            |expected: &DnsRecord, conn_type: XmppConnectionType| -> DnsRecordCheckResult {
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
}
