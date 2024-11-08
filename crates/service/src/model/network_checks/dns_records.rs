// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use async_trait::async_trait;
use tracing::debug;

use crate::{
    model::{
        dns::{DnsEntry, DnsRecord},
        xmpp::XmppConnectionType,
    },
    services::network_checker::{DnsLookupError, NetworkChecker},
};

use super::{NetworkCheck, RetryableNetworkCheckResult};

/// NOTE: This is a `struct` because [`DnsEntry`] is an enum (see [`DnsEntry`]'s doc).
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

#[async_trait]
impl NetworkCheck for DnsRecordCheck {
    type CheckResult = DnsRecordCheckResult;

    fn description(&self) -> String {
        self.dns_entry.description()
    }
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult {
        network_checker
            .check_dns_entry(self.dns_entry.clone())
            .await
    }
}

impl NetworkChecker {
    async fn check_dns_entry(&self, dns_entry: DnsEntry) -> DnsRecordCheckResult {
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

        macro_rules! check_ipv4 {
            ($expected:expr) => {{
                let expected = $expected;
                let host = expected.hostname().to_string();
                check(self.ipv4_lookup(&host).await, expected)
            }};
        }
        macro_rules! check_ipv6 {
            ($expected:expr) => {{
                let expected = $expected;
                let host = expected.hostname().to_string();
                check(self.ipv6_lookup(&host).await, expected)
            }};
        }
        macro_rules! check_srv {
            ($expected:expr, $conn_type:expr) => {{
                let expected = $expected;
                let host = expected.hostname();
                let result = match self
                    .srv_lookup(&$conn_type.standard_domain(host.clone()).to_string())
                    .await
                {
                    Ok(res) => Ok(res),
                    Err(_) => self.srv_lookup(&host.to_string()).await,
                };
                check(result.map(|res| res.records), expected)
            }};
        }

        match dns_entry {
            DnsEntry::Ipv4 { .. } => check_ipv4!(&dns_entry.into_dns_record()),
            DnsEntry::Ipv6 { .. } => check_ipv6!(&dns_entry.into_dns_record()),
            DnsEntry::SrvC2S { .. } => {
                check_srv!(&dns_entry.into_dns_record(), XmppConnectionType::C2S)
            }
            DnsEntry::SrvS2S { .. } => {
                check_srv!(&dns_entry.into_dns_record(), XmppConnectionType::S2S)
            }
        }
    }
}
