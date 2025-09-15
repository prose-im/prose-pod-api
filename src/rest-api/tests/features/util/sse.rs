// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use cucumber::gherkin::Step;
use lazy_static::lazy_static;
use regex::Regex;

use crate::features::prelude::*;

// MARK: - Then

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

#[then(expr = "one SSE with id {string} is")]
async fn then_sse_event(world: &mut TestWorld, id: String, step: &Step) {
    let value = step.docstring().unwrap().trim().to_owned();

    let events = received_events(world);

    let expected = value
        // Unescape double quotes
        .replace(r#"\""#, r#"""#)
        // Unescape newlines
        .replace("\\n", "\n");
    let mut expected = (expected.lines())
        .map(ToOwned::to_owned)
        .collect::<HashSet<String>>();
    expected.insert(format!("id:{id}"));

    assert!(
        events.iter().any(|set| set == &expected),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}

#[then(expr = "at least one SSE has id {string}")]
async fn then_sse_event_id(world: &mut TestWorld, id: String) {
    let events = received_events(world);
    let expected = format!("id:{id}");

    assert!(
        events.iter().any(|set| set.contains(&expected)),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}

#[then(expr = "no SSE has id {string}")]
async fn then_no_sse_event_id(world: &mut TestWorld, id: String) {
    let events = received_events(world);
    let expected = format!("id:{id}");

    assert!(
        !events.iter().any(|set| set.contains(&expected)),
        "events: {events:#?}\nexpected: {expected:#?}"
    );
}
