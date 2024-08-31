// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use cucumber::{given, then, when};
use prose_pod_api::{
    error::Error,
    v1::network::dns::{DnsRecordDiscriminants, GetDnsRecordsResponse},
};
use service::{
    entity::pod_config,
    repositories::{PodConfigCreateForm, PodConfigRepository},
    sea_orm::{ActiveModelTrait as _, EntityTrait, IntoActiveModel as _, ModelTrait, Set},
};

use crate::{api_call_fn, user_token, TestWorld};

api_call_fn!(get_dns_instructions, get, "/v1/network/dns/records");

async fn given_pod_config(
    world: &mut TestWorld,
    update: impl FnOnce(
        &mut <<pod_config::Model as ModelTrait>::Entity as EntityTrait>::ActiveModel,
    ) -> (),
    create: impl FnOnce() -> PodConfigCreateForm,
) -> Result<(), Error> {
    let db = world.db();
    if let Ok(Some(model)) = PodConfigRepository::get(db).await {
        let mut pod_config = model.into_active_model();
        update(&mut pod_config);
        pod_config.update(world.db()).await?;
    } else {
        PodConfigRepository::create(db, create()).await?;
    }

    Ok(())
}

#[given("the Prose Pod is publicly accessible via an IPv4")]
async fn given_pod_has_ipv4(world: &mut TestWorld) -> Result<(), Error> {
    // A random IPv4, as we don't care about the value.
    let ipv4 = "106.142.13.9".to_string();
    given_pod_config(
        world,
        |pod_config| pod_config.ipv4 = Set(Some(ipv4.clone())),
        || PodConfigCreateForm {
            ipv4: Some(ipv4.clone()),
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[given("the Prose Pod is publicly accessible via an IPv6")]
async fn given_pod_has_ipv6(world: &mut TestWorld) -> Result<(), Error> {
    // A random IPv6, as we don't care about the value.
    let ipv6 = "758a:effa:3705:0e60:e681:8514:09c2:3a9a".to_string();
    given_pod_config(
        world,
        |pod_config| pod_config.ipv6 = Set(Some(ipv6.clone())),
        || PodConfigCreateForm {
            ipv6: Some(ipv6.clone()),
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[given("the Prose Pod is publicly accessible via a hostname")]
async fn given_pod_has_hostname(world: &mut TestWorld) -> Result<(), Error> {
    // A random IPv6, as we don't care about the value.
    let hostname = "test.prose.org".to_string();
    given_pod_config(
        world,
        |pod_config| pod_config.hostname = Set(Some(hostname.clone())),
        || PodConfigCreateForm {
            hostname: Some(hostname.clone()),
            ..Default::default()
        },
    )
    .await?;
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

#[then(expr = "step {int} should contain a single {} record")]
async fn then_step_n_single_record(world: &mut TestWorld, n: usize, record_type: String) {
    let res: GetDnsRecordsResponse = world.result().body_into();
    let step = res.steps.get(n).expect(&format!("No step {n}."));
    assert_eq!(step.records.len(), 1);
    let record_type = DnsRecordDiscriminants::from_str(&record_type).expect("Invalid record type.");
    assert_eq!(step.records[0].record_type(), record_type);
}
