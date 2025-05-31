// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;
use tracing::error;

use crate::{features::prelude::*, test_api::test_server};

#[given(expr = "the Prose Pod API {state_verb} started")]
async fn given_api_run_state(world: &mut TestWorld, state: parameters::StateVerb) {
    if state.into_bool() {
        assert!(world.api.is_none());
        world.api = Some(test_server(&world).await.unwrap());
        world.reset_server_ctl_counts();
    } else {
        assert!(world.api.is_none());
        world.api = None;
    }
}

#[when("the Prose Pod API starts")]
async fn when_api_starts(world: &mut TestWorld) {
    assert!(world.api.is_none());
    match test_server(&world).await {
        Ok(server) => world.api = Some(server),
        Err(err) => {
            world.api = None;
            error!("Startup error: {err}");
        }
    }
}

#[then(expr = "the Prose Pod API {state_verb} running")]
async fn then_api_run_state(world: &mut TestWorld, state: parameters::StateVerb) {
    if state.into_bool() {
        assert!(world.api.is_some());
    } else {
        assert!(world.api.is_none());
    }
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

#[then("the call should succeed")]
fn then_response_success(world: &mut TestWorld) {
    world.result().assert_status_success();
}

#[then("the call should not succeed")]
#[then("the call should fail")]
fn then_response_failure(world: &mut TestWorld) {
    world.result().assert_status_failure();
}

#[then("the response content type should be JSON")]
fn then_content_type_json(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(
        res.maybe_content_type(),
        Some("application/json".to_owned())
    );
}

#[then(expr = "the response content type should be {string}")]
fn then_content_type(world: &mut TestWorld, content_type: String) {
    let res = world.result();
    assert_eq!(res.maybe_content_type(), Some(content_type));
}

#[then(expr = "the HTTP status code should be {status}")]
fn then_response_http_status(world: &mut TestWorld, status: parameters::HTTPStatus) {
    world.result().assert_status(*status);
}

#[then(expr = "the response should contain a {string} HTTP header")]
fn then_response_headers_contain(world: &mut TestWorld, header_name: String) {
    world.result().assert_contains_header(header_name);
}

#[then(expr = "the {string} header should contain {string}")]
fn then_response_header_equals(world: &mut TestWorld, header_name: String, header_value: String) {
    world.result().assert_header(header_name, header_value);
}

#[then("the response is a SSE stream")]
fn then_response_is_sse_stream(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(
        res.maybe_content_type(),
        Some("text/event-stream".to_owned()),
        "Content type (body: {:#?})",
        res.text()
    );
}

lazy_static! {
    static ref SEMICOLON_SPACE_REGEX: Regex = Regex::new(r"(?m)(^.*:)\s").unwrap();
}

fn received_events(world: &mut TestWorld) -> Vec<HashSet<String>> {
    (world.result().text())
        .split("\n\n")
        // Remove spaces after semicolons (`:`)
        .map(|s| SEMICOLON_SPACE_REGEX.replace_all(&s, "$1").to_string())
        .map(|s| s.lines().map(ToOwned::to_owned).collect::<HashSet<_>>())
        .collect::<Vec<_>>()
}

#[then(expr = "one SSE event is {string}")]
async fn then_sse_event(world: &mut TestWorld, value: String) {
    let events = received_events(world);

    let expected = value
        // Unescape double quotes
        .replace(r#"\""#, r#"""#)
        // Unescape newlines
        .replace("\\n", "\n");
    let expected = (expected.lines())
        .map(ToOwned::to_owned)
        .collect::<HashSet<String>>();

    assert!(
        events.iter().any(|set| set == &expected),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}

#[then(expr = "at least one SSE event has id {string}")]
async fn then_sse_event_id(world: &mut TestWorld, id: String) {
    let events = received_events(world);
    let expected = format!("id:{id}");

    assert!(
        events.iter().any(|set| set.contains(&expected)),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}

#[then(expr = "no SSE event has id {string}")]
async fn then_no_sse_event_id(world: &mut TestWorld, id: String) {
    let events = received_events(world);
    let expected = format!("id:{id}");

    assert!(
        !events.iter().any(|set| set.contains(&expected)),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}

#[then(expr = "<{jid}>'s password is changed")]
fn then_password_changed(world: &mut TestWorld, jid: parameters::JID) {
    assert_ne!(world.mock_secrets_store.changes_count(&jid), 0);
}
