// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::{Arc, RwLock},
};

use cucumber::given;
use hickory_proto::rr::Name as HickoryDomainName;
use service::network_checks::{
    DnsLookupError, DnsRecord, DnsRecordDiscriminants, NetworkCheckerImpl, SrvLookupResponse,
};
use tracing::trace;

use crate::{
    cucumber_parameters::{DomainName, OpenState},
    TestWorld,
};

#[derive(Debug, Clone, Default)]
pub struct MockNetworkChecker {
    dns_zone: Arc<RwLock<Vec<DnsRecord>>>,
    open_ports: Arc<RwLock<HashMap<HickoryDomainName, HashSet<u16>>>>,
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
        trace!("Checking if port {port_number} is open for {host}…");
        let mut host =
            HickoryDomainName::from_str(host).expect(&format!("Invalid domain name: {host}"));
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

#[given(expr = "{domain_name}'s DNS zone has no {} record for {domain_name}")]
#[given(expr = "{domain_name}’s DNS zone has no {} record for {domain_name}")]
async fn given_no_record(
    _world: &mut TestWorld,
    _host: DomainName,
    _record_type: String,
    _record_hostname: DomainName,
) {
    // Nothing to do as the state is empty when each scenario starts
}

fn add_record(world: &mut TestWorld, dns_record: DnsRecord) {
    let dns_zone = world.mock_network_checker.dns_zone.clone();
    dns_zone.write().unwrap().push(dns_record);
    // println!("Added record. All records: {:#?}", dns_zone.read().unwrap());
}

#[given(expr = "{domain_name}'s DNS zone has a A record for {domain_name}")]
#[given(expr = "{domain_name}’s DNS zone has a A record for {domain_name}")]
async fn given_a_record(world: &mut TestWorld, _host: DomainName, record_hostname: DomainName) {
    add_record(
        world,
        DnsRecord::A {
            hostname: record_hostname.into(),
            ttl: 42,
            value: Ipv4Addr::UNSPECIFIED,
        },
    );
}

#[given(expr = "{domain_name}'s DNS zone has a AAAA record for {domain_name}")]
#[given(expr = "{domain_name}’s DNS zone has a AAAA record for {domain_name}")]
async fn given_aaaa_record(world: &mut TestWorld, _host: DomainName, record_hostname: DomainName) {
    add_record(
        world,
        DnsRecord::AAAA {
            hostname: record_hostname.into(),
            ttl: 42,
            value: Ipv6Addr::UNSPECIFIED,
        },
    );
}

#[given(
    expr = "{domain_name}'s DNS zone has a CNAME record redirecting {domain_name} to {domain_name}"
)]
#[given(
    expr = "{domain_name}’s DNS zone has a CNAME record redirecting {domain_name} to {domain_name}"
)]
async fn given_cname_record(
    world: &mut TestWorld,
    _host: DomainName,
    record_hostname: DomainName,
    target: DomainName,
) {
    add_record(
        world,
        DnsRecord::CNAME {
            hostname: record_hostname.into(),
            ttl: 42,
            target: target.into(),
        },
    );
}

#[given(
    expr = "{domain_name}'s DNS zone has a SRV record for {domain_name} redirecting port {int} to {domain_name}"
)]
#[given(
    expr = "{domain_name}’s DNS zone has a SRV record for {domain_name} redirecting port {int} to {domain_name}"
)]
async fn given_srv_record(
    world: &mut TestWorld,
    _host: DomainName,
    record_hostname: DomainName,
    port: u16,
    record_target: DomainName,
) {
    add_record(
        world,
        DnsRecord::SRV {
            hostname: record_hostname.into(),
            ttl: 42,
            priority: 42,
            weight: 42,
            port,
            target: record_target.into(),
        },
    );
}

#[given(expr = "{domain_name}'s port {int} is {open_or_not}")]
#[given(expr = "{domain_name}’s port {int} is {open_or_not}")]
async fn given_port_open_or_not(
    world: &mut TestWorld,
    host: DomainName,
    port: u16,
    open_or_not: OpenState,
) {
    let mut write_guard = world.mock_network_checker.open_ports.write().unwrap();
    let mut host = host.0;
    host.set_fqdn(true);
    let open_ports = write_guard.entry(host).or_default();
    if open_or_not.as_bool() {
        open_ports.insert(port);
    } else {
        open_ports.remove(&port);
    }
}
