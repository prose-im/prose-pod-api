// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::codegen::anyhow;
use service::onboarding;

use super::prelude::*;

api_call_fn!(get_onboarding_steps_statuses, GET, "/v1/onboarding-steps");

#[given(expr = "onboarding step {string} is {bool}")]
async fn given_onboarding_step(
    world: &mut TestWorld,
    key: String,
    status: parameters::Bool,
) -> anyhow::Result<()> {
    onboarding::KvStore::set_bool(world.db(), &key, *status).await?;
    Ok(())
}

#[when(expr = "{} queries onboarding steps statuses")]
async fn when_get_onboarding_step_statuses(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = get_onboarding_steps_statuses(world.api(), token).await;
    world.result = Some(res.into());
}

#[then(expr = "onboarding step {string} should be {bool}")]
async fn then_onboarding_step(
    world: &mut TestWorld,
    key: String,
    expected: parameters::Bool,
) -> anyhow::Result<()> {
    let status = onboarding::KvStore::get_bool(world.db(), &key).await?;
    assert_eq!(status, Some(*expected));
    Ok(())
}
