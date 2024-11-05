// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use cucumber::{given, then, when};
use prose_pod_api::{error::Error, features::dns_setup::GetDnsRecordsResponse};
use service::{
    features::{
        network_checks::{DnsRecord, DnsRecordDiscriminants},
        pod_config::{entities::pod_config, PodConfigRepository},
    },
    sea_orm::{ActiveModelTrait as _, IntoActiveModel, Set},
};

use crate::{
    api_call_fn,
    cucumber_parameters::{Array, DnsRecordType, DomainName},
    user_token, TestWorld,
};

api_call_fn!(get_dns_instructions, get, "/v1/network/dns/records");

async fn given_pod_config(
    world: &mut TestWorld,
    update: impl FnOnce(&mut pod_config::ActiveModel) -> (),
) -> Result<(), Error> {
    let db = world.db();
    let mut model = PodConfigRepository::get(db)
        .await?
        .map(IntoActiveModel::into_active_model)
        .unwrap_or_default();
    update(&mut model);
    model.save(world.db()).await?;
    Ok(())
}

#[given("the Prose Pod is publicly accessible via an IPv4")]
async fn given_pod_has_ipv4(world: &mut TestWorld) -> Result<(), Error> {
    // We don't care about the value so we can leave it unspecified.
    let ipv4 = Ipv4Addr::UNSPECIFIED.to_string();
    given_pod_config(world, |model| model.ipv4 = Set(Some(ipv4))).await?;
    Ok(())
}

#[given("the Prose Pod is publicly accessible via an IPv6")]
async fn given_pod_has_ipv6(world: &mut TestWorld) -> Result<(), Error> {
    // We don't care about the value so we can leave it unspecified.
    let ipv6 = Ipv6Addr::UNSPECIFIED.to_string();
    given_pod_config(world, |model| model.ipv6 = Set(Some(ipv6))).await?;
    Ok(())
}

#[given("the Prose Pod is publicly accessible via a hostname")]
async fn given_pod_has_hostname(world: &mut TestWorld) -> Result<(), Error> {
    // A random IPv6, as we don't care about the value.
    let hostname = "test.prose.org".to_string();
    given_pod_config(world, |model| model.hostname = Set(Some(hostname))).await?;
    Ok(())
}

#[when(expr = "{} requests DNS setup instructions")]
async fn when_get_dns_instructions(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = get_dns_instructions(world.client(), token).await;
    world.result = Some(res.into());
}

#[then(expr = "DNS setup instructions should contain {int} steps")]
async fn then_dns_instructions_n_steps(world: &mut TestWorld, n: usize) {
    let res: GetDnsRecordsResponse = world.result().body_into();
    assert_eq!(res.steps.len(), n);
}

/// NOTE: Step numbers start at 1.
#[then(expr = "step(s) {int} should contain a single {dns_record_type} record")]
async fn then_step_n_single_record(world: &mut TestWorld, n: usize, record_type: DnsRecordType) {
    let n = n - 1;
    let res: GetDnsRecordsResponse = world.result().body_into();
    let step = res.steps.get(n).expect(&format!("No step {n}."));
    assert_eq!(step.records.len(), 1);
    let record_type = record_type.0;
    assert_eq!(step.records[0].record_type(), record_type);
}

/// NOTE: Step numbers start at 1.
#[then(expr = "step(s) {int} should contain {array} records")]
async fn then_step_n_records(world: &mut TestWorld, n: usize, record_types: Array<DnsRecordType>) {
    let n = n - 1;
    let res: GetDnsRecordsResponse = world.result().body_into();
    let step = res.steps.get(n).expect(&format!("No step {n}."));
    let record_types: Vec<DnsRecordDiscriminants> = record_types.iter().map(|r| r.0).collect();
    let expected: Vec<DnsRecordDiscriminants> =
        step.records.iter().map(|r| r.inner.record_type()).collect();
    assert_eq!(expected, record_types);
}

#[then(expr = "DNS setup instructions should contain a SRV record for port(s) {int}")]
async fn then_srv_record_for_port(world: &mut TestWorld, port_number: u16) {
    let res: GetDnsRecordsResponse = world.result().body_into();
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

#[then(expr = "A records hostnames should be {domain_name}")]
async fn then_a_records_hostnames(world: &mut TestWorld, hostname: DomainName) {
    let res: GetDnsRecordsResponse = world.result().body_into();
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
    let res: GetDnsRecordsResponse = world.result().body_into();
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

#[then(expr = "SRV records hostnames should be {domain_name}")]
async fn then_srv_records_hostnames(world: &mut TestWorld, hostname: DomainName) {
    let res: GetDnsRecordsResponse = world.result().body_into();
    let hostnames: Vec<_> = res
        .steps
        .into_iter()
        .flat_map(|step| step.records)
        .filter_map(|r| match r.inner {
            DnsRecord::SRV { hostname, .. } => Some(hostname),
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
    let res: GetDnsRecordsResponse = world.result().body_into();
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
