// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::features::prelude::*;

// MARK: - Then

#[then("the call should succeed")]
fn then_response_success(world: &mut TestWorld) {
    world.result().assert_status_success();
}

#[then("the call should not succeed")]
#[then("the call should fail")]
fn then_response_failure(world: &mut TestWorld) {
    world.result().assert_status_failure();
}

#[then("the response body should be empty")]
fn then_response_body_empty(world: &mut TestWorld) {
    world.result().as_bytes().is_empty();
}

#[then("the response content type should be JSON")]
fn then_content_type_json(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(
        res.maybe_content_type(),
        Some("application/json".to_owned())
    );
}

#[then(expr = "the response JSON should contain key {string}")]
fn then_response_json_has_key(world: &mut TestWorld, key: String) {
    let map: serde_json::Map<String, serde_json::Value> = world.result().json();
    assert!(map.keys().any(|k| k == &key), "{map:#?}")
}

#[then(expr = "the response JSON should not contain key {string}")]
fn then_response_json_no_key(world: &mut TestWorld, key: String) {
    let map: serde_json::Map<String, serde_json::Value> = world.result().json();
    assert!(!map.keys().any(|k| k == &key), "{map:#?}")
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
