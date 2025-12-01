// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use cucumber::{given, then, when};
use prose_pod_api::{error::Error, features::dns_setup::GetDnsRecordsResponse};
use service::{
    network_checks::{DnsRecord, DnsRecordDiscriminants},
    server_config,
};

use crate::{api_call_fn, cucumber_parameters::*, TestWorld};

// MARK: - Given

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

#[given(expr = "federation is {toggle}")]
async fn given_federation(world: &mut TestWorld, enabled: ToggleState) -> Result<(), Error> {
    server_config::federation_enabled::set(&world.db.write, enabled.as_bool()).await?;
    Ok(())
}

// MARK: - When

api_call_fn!(get_dns_instructions, GET, "/v1/network/dns/records");

#[when(expr = "{} requests DNS setup instructions")]
async fn when_get_dns_instructions(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = get_dns_instructions(world.api(), auth).await;
    world.result = Some(res.unwrap().into());
}

// MARK: - Then

#[then(expr = "DNS setup instructions should contain {int} steps")]
async fn then_dns_instructions_n_steps(world: &mut TestWorld, n: usize) {
    let res: GetDnsRecordsResponse = world.result().json();
    assert_eq!(res.steps.len(), n, "given: {:#?}", res.steps);
}

/// NOTE: Step numbers start at 1.
#[then(expr = "step {int} should contain a single {dns_record_type} record")]
async fn then_step_n_single_record(world: &mut TestWorld, n: usize, record_type: DnsRecordType) {
    then_step_n_records(world, n, Array(vec![record_type])).await;
}

/// NOTE: Step numbers start at 1.
#[then(expr = "step {int} should contain {array} records")]
async fn then_step_n_records(world: &mut TestWorld, n: usize, record_types: Array<DnsRecordType>) {
    let n = n - 1;
    let res: GetDnsRecordsResponse = world.result().json();
    let step = res.steps.get(n).expect(&format!("No step {n}."));
    let record_types: Vec<DnsRecordDiscriminants> = record_types.iter().map(|r| r.0).collect();
    let expected: Vec<DnsRecordDiscriminants> =
        step.records.iter().map(|r| r.inner.record_type()).collect();
    assert_eq!(expected, record_types);
}

#[then(expr = "DNS setup instructions should contain a SRV record for port {int}")]
async fn then_srv_record_for_port(world: &mut TestWorld, port_number: u16) {
    let res: GetDnsRecordsResponse = world.result().json();
    let srv_ports: Vec<u16> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::SRV { port, .. } => Some(port),
            _ => None,
        })
        .collect();
    assert!(srv_ports.contains(&port_number));
}

#[then(expr = "DNS setup instructions should not contain a SRV record for port {int}")]
async fn then_no_srv_record_for_port(world: &mut TestWorld, port_number: u16) {
    let res: GetDnsRecordsResponse = world.result().json();
    let srv_ports: Vec<u16> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::SRV { port, .. } => Some(port),
            _ => None,
        })
        .collect();
    assert!(!srv_ports.contains(&port_number));
}

#[then(expr = "A records hostnames should be {domain_name}")]
async fn then_a_records_hostnames(world: &mut TestWorld, hostname: DomainName) {
    let res: GetDnsRecordsResponse = world.result().json();
    let hostnames: Vec<_> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::A { hostname, .. } => Some(hostname),
            _ => None,
        })
        .collect();
    assert!(
        hostnames.iter().all(|h| h == &hostname),
        "hostnames={hostnames:?}",
    );
}

#[then(expr = "AAAA records hostnames should be {domain_name}")]
async fn then_aaaa_records_hostnames(world: &mut TestWorld, hostname: DomainName) {
    let res: GetDnsRecordsResponse = world.result().json();
    let hostnames: Vec<_> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::AAAA { hostname, .. } => Some(hostname),
            _ => None,
        })
        .collect();
    assert!(
        hostnames.iter().all(|h| h == &hostname),
        "hostnames={hostnames:?}",
    );
}

#[then(expr = "SRV record hostname should be {domain_name} for port {int}")]
async fn then_srv_records_hostnames(world: &mut TestWorld, hostname: DomainName, port_filter: u16) {
    let res: GetDnsRecordsResponse = world.result().json();
    let hostnames: Vec<_> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::SRV { hostname, port, .. } if port == port_filter => Some(hostname),
            _ => None,
        })
        .collect();
    assert!(
        hostnames.iter().all(|h| h == &hostname),
        "hostnames={hostnames:?}",
    );
}

#[then(expr = "SRV records targets should be {domain_name}")]
async fn then_srv_records_targets(world: &mut TestWorld, target: DomainName) {
    let res: GetDnsRecordsResponse = world.result().json();
    let targets: Vec<_> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::SRV { target, .. } => Some(target),
            _ => None,
        })
        .collect();
    assert!(targets.iter().all(|h| h == &target), "targets={targets:?}");
}
