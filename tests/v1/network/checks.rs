// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::when;
use rocket::http::Accept;

use crate::{api_call_fn, user_token, TestWorld};

api_call_fn!(
    check_dns_records,
    get,
    "/v1/network/checks/dns",
    Accept::EventStream
);

#[when(expr = "{} checks the DNS records configuration")]
async fn when_check_dns(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = check_dns_records(world.client(), token).await;
    world.result = Some(res.into());
}

api_call_fn!(
    check_ports,
    get,
    "/v1/network/checks/ports",
    Accept::EventStream
);

#[when(expr = "{} checks the ports reachability")]
async fn when_check_ports(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = check_ports(world.client(), token).await;
    world.result = Some(res.into());
}

api_call_fn!(
    check_ip_connectivity,
    get,
    "/v1/network/checks/ip",
    Accept::EventStream
);

#[when(expr = "{} checks the IP connectivity")]
async fn when_check_ip_connectivity(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = check_ip_connectivity(world.client(), token).await;
    world.result = Some(res.into());
}
