// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::Name as DomainName;
use service::network_checks::{
    DnsLookupError, DnsRecord, DnsRecordDiscriminants, NetworkCheckerImpl, SrvLookupResponse,
};

use super::prelude::*;

#[derive(Debug, Default)]
pub struct MockNetworkChecker {
    pub dns_zone: Arc<RwLock<Vec<DnsRecord>>>,
    pub open_ports: Arc<RwLock<HashMap<DomainName, HashSet<u16>>>>,
}

impl MockNetworkChecker {
    fn lookup_(
        &self,
        domain: &str,
        record_type: DnsRecordDiscriminants,
    ) -> Result<Vec<DnsRecord>, DnsLookupError> {
        // println!("lookup_(domain: {domain}, record_type: {record_type:?})");
        let records: Vec<DnsRecord> = self
            .dns_zone
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|record| {
                record.record_type() == record_type && record.hostname().to_string().eq(domain)
            })
            .collect();
        // println!("filtered records: {:#?}", records);
        if records.is_empty() {
            Err(DnsLookupError(format!(
                "No {} record found",
                <&str>::from(record_type),
            )))
        } else {
            Ok(records)
        }
    }
}

#[async_trait::async_trait]
impl NetworkCheckerImpl for MockNetworkChecker {
    async fn ipv4_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        self.lookup_(domain, DnsRecordDiscriminants::A)
    }
    async fn ipv6_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        self.lookup_(domain, DnsRecordDiscriminants::AAAA)
    }
    async fn srv_lookup(&self, domain: &str) -> Result<SrvLookupResponse, DnsLookupError> {
        self.lookup_(domain, DnsRecordDiscriminants::SRV)
            .map(|records| SrvLookupResponse {
                recursively_resolved_ips: Default::default(),
                srv_targets: records
                    .iter()
                    .map(|rec| match rec {
                        DnsRecord::SRV { target, .. } => target.clone(),
                        _ => panic!(),
                    })
                    .collect(),
                records,
            })
    }
    async fn cname_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        self.lookup_(domain, DnsRecordDiscriminants::CNAME)
    }

    fn is_port_open(&self, host: &str, port_number: u16) -> bool {
        tracing::trace!("Checking if port {port_number} is open for {host}…");
        let mut host = DomainName::from_str(host).expect(&format!("Invalid domain name: {host}"));
        host.set_fqdn(true);
        self.open_ports
            .read()
            .unwrap()
            .get(&host)
            .is_some_and(|vec| vec.contains(&port_number))
    }

    async fn is_ipv4_available(&self, host: &str) -> bool {
        self.lookup_(host, DnsRecordDiscriminants::A)
            .is_ok_and(|vec| !vec.is_empty())
    }
    async fn is_ipv6_available(&self, host: &str) -> bool {
        self.lookup_(host, DnsRecordDiscriminants::AAAA)
            .is_ok_and(|vec| !vec.is_empty())
    }
}
