// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::when;
use prose_pod_api::v1::pod::config::SetPodAddressRequest;

use crate::{api_call_fn, user_token, TestWorld};

api_call_fn!(
    set_pod_address,
    put,
    "/v1/pod/config/address",
    payload: SetPodAddressRequest
);

#[when(expr = "{} sets the Prose Pod address to an IPv4")]
async fn when_set_pod_address_ipv4(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = set_pod_address(
        world.client(),
        token,
        SetPodAddressRequest {
            ipv4: Some("104.18.28.104".to_string()),
            ..Default::default()
        },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the Prose Pod address to an IPv6")]
async fn when_set_pod_address_ipv6(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = set_pod_address(
        world.client(),
        token,
        SetPodAddressRequest {
            ipv6: Some("2606:4700::6812:1c68".to_string()),
            ..Default::default()
        },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the Prose Pod address to a hostname")]
async fn when_set_pod_address_hostname(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = set_pod_address(
        world.client(),
        token,
        SetPodAddressRequest {
            hostname: Some("crisp.chat".to_string()),
            ..Default::default()
        },
    )
    .await;
    world.result = Some(res.into());
}
