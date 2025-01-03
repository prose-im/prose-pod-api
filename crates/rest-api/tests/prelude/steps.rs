// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use lazy_static::lazy_static;
use regex::Regex;

use crate::{features::prelude::*, test_api::test_server};

#[given("the Prose Pod API has started")]
async fn given_api_started(world: &mut TestWorld) {
    assert!(world.api.is_none());
    world.api = Some(test_server(&world).await);
    world.reset_server_ctl_counts();
}

#[when("the Prose Pod API starts")]
async fn when_api_starts(world: &mut TestWorld) {
    assert!(world.api.is_none());
    world.api = Some(test_server(&world).await);
}

#[given("the XMPP server is offline")]
fn given_xmpp_server_offline(world: &mut TestWorld) {
    world.xmpp_service_state_mut().online = false;
    world.server_ctl_state_mut().online = false;
}

#[then("the call should succeed")]
fn then_response_success(world: &mut TestWorld) {
    world.result().assert_status_success();
}

#[then("the call should not succeed")]
fn then_response_failure(world: &mut TestWorld) {
    world.result().assert_status_failure();
}

#[then("the response content type should be JSON")]
fn then_response_json(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(
        res.maybe_content_type(),
        Some("application/json".to_owned())
    );
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
        Some("text/event-stream".to_owned())
    );
}

lazy_static! {
    static ref UNEXPECTED_SEMICOLON_REGEX: Regex = Regex::new(r"(\n|^):(\n|$)").unwrap();
    static ref UNEXPECTED_NEWLINE_REGEX: Regex = Regex::new(r"\n$").unwrap();
}

#[then(expr = "one SSE event is {string}")]
async fn then_sse_event(world: &mut TestWorld, value: String) {
    let res = world.result();
    let events = res
        .text()
        .split("\n\n")
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    let expected = value
        // Unescape double quotes
        .replace(r#"\""#, r#"""#)
        // Unescape newlines
        .replace("\\n", "\n");
    assert!(
        events.contains(&expected),
        "events: {events:#?}\nexpected: {expected:?}"
    );
}

#[then(expr = "<{jid}>'s password is changed")]
fn then_password_changed(world: &mut TestWorld, jid: parameters::JID) {
    assert_ne!(world.mock_secrets_store.changes_count(&jid), 0);
}
