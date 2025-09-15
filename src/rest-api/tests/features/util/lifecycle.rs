// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::error;

use crate::{features::prelude::*, test_api::test_server};

// MARK: - Given

pub async fn given_api_started(world: &mut TestWorld) {
    assert!(world.api.is_none());
    world.api = Some(test_server(world).await.unwrap());
    world.reset_server_ctl_counts();
}

#[given(expr = "the Prose Pod API {state_verb} started")]
async fn given_api_run_state(world: &mut TestWorld, state: parameters::StateVerb) {
    if state.into_bool() {
        given_api_started(world).await;
    } else {
        assert!(world.api.is_none());
        world.api = None;
    }
}

#[given(expr = "the Prose Pod API has restarted")]
async fn given_api_restarted(world: &mut TestWorld) {
    world.api = Some(test_server(world).await.unwrap());
    world.reset_server_ctl_counts();
}

#[given("the XMPP server is offline")]
fn given_xmpp_server_offline(world: &mut TestWorld) {
    world.xmpp_service_state_mut().online = false;
    world.server_ctl_state_mut().online = false;
}

#[given(expr = "the SMTP server {state_verb} reachable")]
fn given_smtp_server_offline(world: &mut TestWorld, state: parameters::StateVerb) {
    world.email_notifier_state_mut().online = state.into_bool();
}

// MARK: - When

#[when("the Prose Pod API starts")]
async fn when_api_starts(world: &mut TestWorld) {
    assert!(world.api.is_none());
    match test_server(world).await {
        Ok(server) => world.api = Some(server),
        Err(err) => {
            world.api = None;
            error!("Startup error: {err}");
        }
    }
}

// MARK: - Then

#[then(expr = "the Prose Pod API {state_verb} running")]
async fn then_api_run_state(world: &mut TestWorld, state: parameters::StateVerb) {
    if state.into_bool() {
        assert!(world.api.is_some());
    } else {
        assert!(world.api.is_none());
    }
}
