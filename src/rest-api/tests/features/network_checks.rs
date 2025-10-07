// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

// MARK: - When

api_call_fn!(check_dns_records, GET, "/v1/network/checks/dns");
api_call_fn!(
    check_dns_records_stream,
    GET,
    "/v1/network/checks/dns",
    accept: "text/event-stream"
);

#[when(expr = "{} checks the DNS records configuration")]
async fn when_check_dns(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = check_dns_records(world.api(), auth).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} checks the DNS records configuration as \"text\\/event-stream\"")]
async fn when_check_dns_stream(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    match check_dns_records_stream(world.api(), auth).await {
        Ok(res) => world.result = Some(res.into()),
        Err(_) => panic!(
            "DNS check failed. Expected: {:#?}",
            (world.pod_network_config().await)
                .dns_setup_steps()
                .flat_map(|step| step.records)
                .map(|record| record.string_repr)
                .collect::<Vec<_>>()
        ),
    }
}

api_call_fn!(
    check_ports,
    GET,
    "/v1/network/checks/ports",
    accept: "text/event-stream"
);

#[when(expr = "{} checks the ports reachability")]
async fn when_check_ports(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = check_ports(world.api(), auth).await;
    world.result = Some(res.unwrap().into());
}

api_call_fn!(
    check_ip_connectivity,
    GET,
    "/v1/network/checks/ip",
    accept: "text/event-stream"
);

#[when(expr = "{} checks the IP connectivity")]
async fn when_check_ip_connectivity(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = check_ip_connectivity(world.api(), auth).await;
    world.result = Some(res.unwrap().into());
}
